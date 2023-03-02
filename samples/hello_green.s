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

.segment "CODE"
.proc irq_handler
  RTI
.endproc

.proc nmi_handler
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
forever:
  JMP forever
.endproc

.segment "VECTORS"
.addr nmi_handler, reset_handler, irq_handler

.segment "CHARS"
.res 8192
.segment "STARTUP"
