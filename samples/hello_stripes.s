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

NAMETABLE_TL = $2000
NAMETABLE_TR = $2400
NAMETABLE_BL = $2800
NAMETABLE_BR = $2C00

PALETTE = $3f00

.segment "CODE"
.proc irq_handler
  RTI
.endproc

.proc nmi_handler
  RTI
.endproc

.macro set_ppu_addr_x addr
  LDX #>(addr)
  STX PPUADDR
  LDX #<(addr)
  STX PPUADDR
.endmacro

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

  set_ppu_addr_x PALETTE
  LDA #$29 ; background colour to green
  STA PPUDATA
  LDA #$11 ; blue
  STA PPUDATA
  LDA #$23 ; purple
  STA PPUDATA
  LDA #$27 ; orange
  STA PPUDATA

  set_ppu_addr_x NAMETABLE_TL

  LDX #80
  LDA #1
  LDY #2
  JSR draw_stripes

  LDX #64
  LDA #3
  LDY #1
  JSR draw_stripes

  ; Enable full backgrounds and sprites, greyscale off, colour emphasis off
  LDA #%00011110
  STA PPUMASK
forever:
  JMP forever
.endproc

.proc draw_stripes
; X is the loop count, A & Y are the tile numbers
loop:
  STA PPUDATA
  STY PPUDATA
  DEX
  BNE loop
  RTS
.endproc

.segment "VECTORS"
.addr nmi_handler, reset_handler, irq_handler

.segment "CHARS"
; 8x8 - 1 tile
.byte $00, $00, $00, $00, $00, $00, $00, $00
.byte $00, $00, $00, $00, $00, $00, $00, $00
; 8x8 - 1 tile
.byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
.byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
; 8x8 - 1 tile
.byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
.byte $00, $00, $00, $00, $00, $00, $00, $00
; 8x8 - 1 tile
.byte $00, $00, $00, $00, $00, $00, $00, $00
.byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
.res 8192 - 16*4
.segment "STARTUP"
