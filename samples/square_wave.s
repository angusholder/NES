.segment "HEADER"
.byte $4e, $45, $53, $1a, $02, $01, $00, $00

PPUCTRL = $2000
PPUMASK = $2001
PPUSTATUS = $2002
OAMADDR = $2003
OAMDATA = $2004
PPUSCROLL = $2005
PPUADDR = $2006
PPUDATA = $2007

.include "nes.inc"

.segment "ZEROPAGE"
BUTTON_A = %00000001
BUTTON_B = %00000010
buttons: .res 1
prev_buttons: .res 1

.segment "CODE"
.proc irq_handler
  RTI
.endproc

.proc nmi_handler
  BIT PPUSTATUS
  BPL not_vblank
  JMP vblank_handler

not_vblank:
  RTI
.endproc

; Called 60 times per second
.proc vblank_handler
  JSR handle_input
  JSR update_sound_control
  RTI
.endproc

.proc reset_handler
  SEI
  CLD
  LDX #$00
  STX PPUCTRL
  STX PPUMASK
vblankwait:
  BIT PPUSTATUS
  BPL vblankwait
  JMP main
.endproc

.proc main
  ; Reset PPUADDR
  LDX PPUSTATUS
  ; Universal background colour
  LDX #$3f
  STX PPUADDR
  LDX #$00
  STX PPUADDR
  ; Set background colour to green
  LDA #$29
  STA PPUDATA
  ; Enable full backgrounds and sprites, greyscale off, colour emphasis off
  LDA #%00011110
  STA PPUMASK

  LDA #%00000001 ; Enable pulse 1
  STA APU_CHANCTRL

  LDA #%10111111 ; length counter halt, constant volume, volume 15
  STA APU_PULSE1CTRL

  LDX #45
  JSR set_pulse1_tone

  LDA #$80 ; Enable vblank NMI
  STA PPUCTRL

forever:
  JMP forever
.endproc

.proc set_pulse1_tone
  LDA frequency_lut_lower,X
  STA APU_PULSE1FTUNE
  LDA frequency_lut_upper,X
  STA APU_PULSE1CTUNE
  RTS
.endproc

frequency_lut_upper:
.byte $07,$07,$07,$06,$06,$05,$05,$05,$05,$04,$04,$04,$03,$03,$03,$03,$03,$02,$02,$02,$02,$02,$02,$02,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01,$01,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
frequency_lut_lower:
.byte $F0,$7C,$10,$AC,$4C,$F2,$9E,$4C,$01,$B8,$74,$34,$F8,$BE,$88,$56,$26,$F9,$CF,$A6,$80,$5C,$3A,$1A,$FC,$DF,$C4,$AB,$93,$7C,$67,$53,$40,$2E,$1D,$0D,$FE,$EF,$E2,$D5,$C9,$BE,$B3,$A9,$A0,$97,$8E,$86,$7E,$77,$71,$6A,$64,$5F,$59,$54,$50,$4B,$47,$43,$3F,$3B,$38,$35,$32,$2F,$2C,$2A,$28,$26,$24,$22,$20,$1E,$1C

.proc handle_input
    LDA buttons
    STA prev_buttons
    LDA #0
    STA buttons

    LDA #1
    STA APU_PAD1
    LDA #0
    STA APU_PAD1

    LDA APU_PAD1
    AND #1
    BEQ input_no_a
    LDA buttons
    ORA #BUTTON_A
    STA buttons

  input_no_a:

    LDA APU_PAD1
    AND #1
    BEQ input_no_b
    LDA buttons
    ORA #BUTTON_B
    STA buttons

  input_no_b:
    RTS
.endproc

.proc update_sound_control
    LDX #35

    LDA buttons
    AND BUTTON_A
    BEQ no_a
    LDX #25
    JMP no_b
  no_a:

    LDA buttons
    AND BUTTON_B
    BEQ no_b
    LDX #45
  no_b:

    JSR set_pulse1_tone
    RTS
.endproc

.segment "VECTORS"
.addr nmi_handler, reset_handler, irq_handler

.segment "CHARS"
.res 8192
.segment "STARTUP"
