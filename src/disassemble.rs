use crate::nes::NES;

use std::fmt::Write;

static INSTRUCTION_NAMES: [&str; 256] = [
    "BRK", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO",
    "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
    "BPL", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO",
    "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
    "JSR", "AND", "KIL", "RLA", "BIT", "AND", "ROL", "RLA",
    "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
    "BMI", "AND", "KIL", "RLA", "NOP", "AND", "ROL", "RLA",
    "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
    "RTI", "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE",
    "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
    "BVC", "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE",
    "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
    "RTS", "ADC", "KIL", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
    "BVS", "ADC", "KIL", "RRA", "NOP", "ADC", "ROR", "RRA",
    "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
    "NOP", "STA", "NOP", "SAX", "STY", "STA", "STX", "SAX",
    "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX", "SAX",
    "BCC", "STA", "KIL", "AHX", "STY", "STA", "STX", "SAX",
    "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "AHX",
    "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "TAY", "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "BCS", "LDA", "KIL", "LAX", "LDY", "LDA", "LDX", "LAX",
    "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
    "CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP",
    "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "KIL", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC",
    "INX", "SBC", "NOP", "SBC", "CPX", "SBC", "INC", "ISC",
    "BEQ", "SBC", "KIL", "ISC", "NOP", "SBC", "INC", "ISC",
    "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

static INSTRUCTION_ADDRESS_MODES: [u8; 256] = [
    /*      0x0  0x1  0x2  0x3  0x4  0x5  0x6  0x7  0x8  0x9  0xA  0xB  0xC  0xD  0xE  0xF*/
    /*0x0*/ 0x5, 0x6, 0x5, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x3, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0x1*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
    /*0x2*/ 0x0, 0x6, 0x5, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x3, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0x3*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
    /*0x4*/ 0x5, 0x6, 0x5, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x3, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0x5*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
    /*0x6*/ 0x5, 0x6, 0x5, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x3, 0x4, 0x7, 0x0, 0x0, 0x0,
    /*0x7*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
    /*0x8*/ 0x4, 0x6, 0x4, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x5, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0x9*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xC, 0xC, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x2, 0x2,
    /*0xA*/ 0x4, 0x6, 0x4, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x5, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0xB*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xC, 0xC, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x2, 0x2,
    /*0xC*/ 0x4, 0x6, 0x4, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x5, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0xD*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
    /*0xE*/ 0x4, 0x6, 0x4, 0x6, 0xA, 0xA, 0xA, 0xA, 0x5, 0x4, 0x5, 0x4, 0x0, 0x0, 0x0, 0x0,
    /*0xF*/ 0x9, 0x8, 0x5, 0x8, 0xB, 0xB, 0xB, 0xB, 0x5, 0x2, 0x5, 0x2, 0x1, 0x1, 0x1, 0x1,
];

const ADDR_ABSOLUTE: u8 = 0x0;
const ADDR_ABSOLUTE_X: u8 = 0x1;
const ADDR_ABSOLUTE_Y: u8 = 0x2;
const ADDR_ACCUMULATOR: u8 = 0x3;
const ADDR_IMMEDIATE: u8 = 0x4;       // #Oper
const ADDR_IMPLIED: u8 = 0x5;
const ADDR_INDEXED_INDIRECT: u8 = 0x6; // (Indirect, X)
const ADDR_INDIRECT: u8 = 0x7;
const ADDR_INDIRECT_INDEXED: u8 = 0x8; // (Indirect), Y
const ADDR_RELATIVE: u8 = 0x9;
const ADDR_ZERO_PAGE: u8 = 0xA;
const ADDR_ZERO_PAGE_X: u8 = 0xB;
const ADDR_ZERO_PAGE_Y: u8 = 0xC;

/*
Trace format, displays registers as they are *before* running the instruction.
Status reg flags are capitalised if set, lower case if clear.
Instructions are indented using the SP's difference from $FF, ie: it indents upon each JSR/interrupt/push, and de-dents upon each RTS/RTI/pop.

c226879701   A:FC X:FF Y:FC S:F7 P:NvUbdIzc         $AF19: 20 D4 BE  JSR $BED4
c226879707   A:FC X:FF Y:FC S:F5 P:NvUbdIzc           $BED4: A2 01     LDX #$01
c226879709   A:FC X:01 Y:FC S:F5 P:nvUbdIzc           $BED6: 86 08     STX $08 = #$05
c226879712   A:FC X:01 Y:FC S:F5 P:nvUbdIzc           $BED8: AD 01 03  LDA $0301 = #$00
c226879716   A:00 X:01 Y:FC S:F5 P:nvUbdIZc           $BEDB: D0 21     BNE $BEFE
*/
#[allow(non_snake_case)]
pub fn disassemble(nes: &mut NES) {
    let savepoint = nes.save_cycles();
    let mut output = String::with_capacity(100);

    let A = nes.A;
    let X = nes.X;
    let Y = nes.Y;
    let SP = nes.SP;
    let SR = nes.SR.clone();
    let PC = nes.PC;

    let op = nes.read8(PC);
    let op_name = INSTRUCTION_NAMES[op as usize];
    let addr_mode = INSTRUCTION_ADDRESS_MODES[op as usize];
    let size = get_addr_mode_instruction_size(addr_mode);
    let mut op_bytes: Vec<u8> = Vec::with_capacity(3);
    for i in 0..size { op_bytes.push(nes.read8(PC + i as u16)); }

    write!(output, "c{:08}  ", nes.get_cycles()).unwrap();
    write!(output, "A:{A:02X} X:{X:02X} Y:{Y:02X} S:{SP:02X} {SR} ").unwrap();

    let indent = 255 - nes.SP;
    for _ in 0..indent { output.push(' '); }
    write!(output, "${PC:04X}: ").unwrap();
    for byte in op_bytes {
        write!(output, "{byte:02X} ").unwrap();
    }
    // Pad out to 3 bytes-worth of hex
    for _ in 0..(3 - size) { output.push_str("   "); }
    // Extra space between bytes and instruction name
    output.push(' ');

    match addr_mode {
        ADDR_ABSOLUTE => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} ${addr:04X}").unwrap();
        }
        ADDR_ABSOLUTE_X => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} ${addr:04X},X").unwrap();
        }
        ADDR_ABSOLUTE_Y => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} ${addr:04X},Y").unwrap();
        }
        ADDR_ACCUMULATOR => {
            write!(output, "{op_name}").unwrap();
        }
        ADDR_IMMEDIATE => {
            let arg = nes.read8(PC+1);
            write!(output, "{op_name} #${arg:02X}").unwrap();
        }
        ADDR_IMPLIED => {
            write!(output, "{op_name}").unwrap();
        }
        ADDR_INDEXED_INDIRECT => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} (${addr:04X},X)").unwrap();
        }
        // JMP_INDIR
        ADDR_INDIRECT => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} (${addr:04X})").unwrap();
        }
        ADDR_INDIRECT_INDEXED => {
            let addr = nes.read_addr(PC+1);
            write!(output, "{op_name} (${addr:04X}),Y").unwrap();
        }
        ADDR_RELATIVE => {
            let offset = nes.read8(PC+1) as i8 as i16;
            let target = (PC+1).wrapping_add_signed(offset);
            write!(output, "{op_name} ${target:04X}").unwrap();
        }
        ADDR_ZERO_PAGE => {
            let addr = nes.read8(PC+1) as u16;
            write!(output, "{op_name} ${addr:04X}").unwrap();
        }
        ADDR_ZERO_PAGE_X => {
            let addr = nes.read8(PC+1) as u16;
            write!(output, "{op_name} ${addr:04X},X").unwrap();
        }
        ADDR_ZERO_PAGE_Y => {
            let addr = nes.read8(PC+1) as u16;
            write!(output, "{op_name} ${addr:04X},Y").unwrap();
        }
        _ => unreachable!(),
    }
    assert_eq!(nes.PC, PC);

    output.push('\n');
    if let Some(output_writer) = nes.trace_output.as_mut() {
        output_writer.write(output.as_bytes()).unwrap();
    }

    nes.restore_cycles(savepoint);
}

fn get_addr_mode_instruction_size(addr_mode: u8) -> usize {
    match addr_mode {
        ADDR_ABSOLUTE | ADDR_ABSOLUTE_X | ADDR_ABSOLUTE_Y => 3,
        ADDR_ACCUMULATOR => 1,
        ADDR_IMMEDIATE => 2,
        ADDR_IMPLIED => 1,
        ADDR_INDEXED_INDIRECT => 3,
        ADDR_INDIRECT => 3,
        ADDR_INDIRECT_INDEXED => 3,
        ADDR_RELATIVE => 2,
        ADDR_ZERO_PAGE | ADDR_ZERO_PAGE_X | ADDR_ZERO_PAGE_Y => 2,
        _ => unreachable!(),
    }
}
