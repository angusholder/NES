use log::{trace};
use crate::nes::{NES, StatusRegister};
use crate::cpu_ops::*;
use crate::disassemble;

#[inline(never)]
pub fn emulate_instruction(nes: &mut NES) {
    if nes.trace_instructions {
        log_instruction(nes);
    }

    let op = nes.read_code();

    match op {
        ADC_IMM => adc(nes, addressing_immediate_rd),
        ADC_ZP => adc(nes, addressing_zeropage_rd),
        ADC_ZPX => adc(nes, addressing_zeropage_x_rd),
        ADC_ABS => adc(nes, addressing_absolute_rd),
        ADC_ABSX => adc(nes, addressing_absolute_x_rd),
        ADC_ABSY => adc(nes, addressing_absolute_y_rd),
        ADC_INDIRX => adc(nes, addressing_indirect_x_rd),
        ADC_INDIRY => adc(nes, addressing_indirect_y_rd),

        SBC_IMM | 0xEB => sbc(nes, addressing_immediate_rd),
        SBC_ZP => sbc(nes, addressing_zeropage_rd),
        SBC_ZPX => sbc(nes, addressing_zeropage_x_rd),
        SBC_ABS => sbc(nes, addressing_absolute_rd),
        SBC_ABSX => sbc(nes, addressing_absolute_x_rd),
        SBC_ABSY => sbc(nes, addressing_absolute_y_rd),
        SBC_INDIRX => sbc(nes, addressing_indirect_x_rd),
        SBC_INDIRY => sbc(nes, addressing_indirect_y_rd),

        AND_IMM => and(nes, addressing_immediate_rd),
        AND_ZP => and(nes, addressing_zeropage_rd),
        AND_ZPX => and(nes, addressing_zeropage_x_rd),
        AND_ABS => and(nes, addressing_absolute_rd),
        AND_ABSX => and(nes, addressing_absolute_x_rd),
        AND_ABSY => and(nes, addressing_absolute_y_rd),
        AND_INDIRX => and(nes, addressing_indirect_x_rd),
        AND_INDIRY => and(nes, addressing_indirect_y_rd),

        EOR_IMM => eor(nes, addressing_immediate_rd),
        EOR_ZP => eor(nes, addressing_zeropage_rd),
        EOR_ZPX => eor(nes, addressing_zeropage_x_rd),
        EOR_ABS => eor(nes, addressing_absolute_rd),
        EOR_ABSX => eor(nes, addressing_absolute_x_rd),
        EOR_ABSY => eor(nes, addressing_absolute_y_rd),
        EOR_INDIRX => eor(nes, addressing_indirect_x_rd),
        EOR_INDIRY => eor(nes, addressing_indirect_y_rd),

        ORA_IMM => ora(nes, addressing_immediate_rd),
        ORA_ZP => ora(nes, addressing_zeropage_rd),
        ORA_ZPX => ora(nes, addressing_zeropage_x_rd),
        ORA_ABS => ora(nes, addressing_absolute_rd),
        ORA_ABSX => ora(nes, addressing_absolute_x_rd),
        ORA_ABSY => ora(nes, addressing_absolute_y_rd),
        ORA_INDIRX => ora(nes, addressing_indirect_x_rd),
        ORA_INDIRY => ora(nes, addressing_indirect_y_rd),

        CMP_IMM => cmp(nes, addressing_immediate_rd),
        CMP_ZP => cmp(nes, addressing_zeropage_rd),
        CMP_ZPX => cmp(nes, addressing_zeropage_x_rd),
        CMP_ABS => cmp(nes, addressing_absolute_rd),
        CMP_ABSX => cmp(nes, addressing_absolute_x_rd),
        CMP_ABSY => cmp(nes, addressing_absolute_y_rd),
        CMP_INDIRX => cmp(nes, addressing_indirect_x_rd),
        CMP_INDIRY => cmp(nes, addressing_indirect_y_rd),

        CPX_IMM => cpx(nes, addressing_immediate_rd),
        CPX_ZP => cpx(nes, addressing_zeropage_rd),
        CPX_ABS => cpx(nes, addressing_absolute_rd),

        CPY_IMM => cpy(nes, addressing_immediate_rd),
        CPY_ZP => cpy(nes, addressing_zeropage_rd),
        CPY_ABS => cpy(nes, addressing_absolute_rd),

        DEX => dex(nes),
        DEY => dey(nes),

        INX => inx(nes),
        INY => iny(nes),

        DEC_ZP => addressing_zeropage_rmw(nes, dec),
        DEC_ZPX => addressing_zeropage_x_rmw(nes, dec),
        DEC_ABS => addressing_absolute_rmw(nes, dec),
        DEC_ABSX => addressing_absolute_x_rmw(nes, dec),

        INC_ZP => addressing_zeropage_rmw(nes, inc),
        INC_ZPX => addressing_zeropage_x_rmw(nes, inc),
        INC_ABS => addressing_absolute_rmw(nes, inc),
        INC_ABSX => addressing_absolute_x_rmw(nes, inc),

        LDA_IMM => lda(nes, addressing_immediate_rd),
        LDA_ZP => lda(nes, addressing_zeropage_rd),
        LDA_ZPX => lda(nes, addressing_zeropage_x_rd),
        LDA_ABS => lda(nes, addressing_absolute_rd),
        LDA_ABSX => lda(nes, addressing_absolute_x_rd),
        LDA_ABSY => lda(nes, addressing_absolute_y_rd),
        LDA_INDIRX => lda(nes, addressing_indirect_x_rd),
        LDA_INDIRY => lda(nes, addressing_indirect_y_rd),

        LDX_IMM => ldx(nes, addressing_immediate_rd),
        LDX_ZP => ldx(nes, addressing_zeropage_rd),
        LDX_ZPY => ldx(nes, addressing_zeropage_y_rd),
        LDX_ABS => ldx(nes, addressing_absolute_rd),
        LDX_ABSY => ldx(nes, addressing_absolute_y_rd),

        LDY_IMM => ldy(nes, addressing_immediate_rd),
        LDY_ZP => ldy(nes, addressing_zeropage_rd),
        LDY_ZPX => ldy(nes, addressing_zeropage_x_rd),
        LDY_ABS => ldy(nes, addressing_absolute_rd),
        LDY_ABSX => ldy(nes, addressing_absolute_x_rd),

        STA_ZP => sta(nes, addressing_zeropage_wr),
        STA_ZPX => sta(nes, addressing_zeropage_x_wr),
        STA_ABS => sta(nes, addressing_absolute_wr),
        STA_ABSX => sta(nes, addressing_absolute_x_wr),
        STA_ABSY => sta(nes, addressing_absolute_y_wr),
        STA_INDIRX => sta(nes, addressing_indirect_x_wr),
        STA_INDIRY => sta(nes, addressing_indirect_y_wr),

        STX_ZP => stx(nes, addressing_zeropage_wr),
        STX_ZPY => stx(nes, addressing_zeropage_y_wr),
        STX_ABS => stx(nes, addressing_absolute_wr),

        STY_ZP => sty(nes, addressing_zeropage_wr),
        STY_ZPX => sty(nes, addressing_zeropage_x_wr),
        STY_ABS => sty(nes, addressing_absolute_wr),

        TAX => { nes.X = nes.A;  update_zn(nes, nes.X); nes.tick(); }
        TAY => { nes.Y = nes.A;  update_zn(nes, nes.Y); nes.tick(); }
        TSX => { nes.X = nes.SP; update_zn(nes, nes.X); nes.tick(); }
        TXA => { nes.A = nes.X;  update_zn(nes, nes.A); nes.tick(); }
        TXS => { nes.SP = nes.X; nes.tick(); }
        TYA => { nes.A = nes.Y;  update_zn(nes, nes.A); nes.tick(); }

        CLC => { nes.SR.C = false; nes.tick(); }
        CLD => { nes.SR.D = false; nes.tick(); }
        CLI => { nes.SR.I = false; nes.tick(); }
        CLV => { nes.SR.V = false; nes.tick(); }
        SEC => { nes.SR.C = true; nes.tick(); }
        SED => { nes.SR.D = true; nes.tick(); }
        SEI => { nes.SR.I = true; nes.tick(); }

        LSR_ZP => addressing_zeropage_rmw(nes, lsr),
        LSR_ZPX => addressing_zeropage_x_rmw(nes, lsr),
        LSR_ABS => addressing_absolute_rmw(nes, lsr),
        LSR_ABSX => addressing_absolute_x_rmw(nes, lsr),
        LSR_ACC => lsr_acc(nes),

        ASL_ZP => addressing_zeropage_rmw(nes, asl),
        ASL_ZPX => addressing_zeropage_x_rmw(nes, asl),
        ASL_ABS => addressing_absolute_rmw(nes, asl),
        ASL_ABSX => addressing_absolute_x_rmw(nes, asl),
        ASL_ACC => asl_acc(nes),

        ROL_ZP => addressing_zeropage_rmw(nes, rol),
        ROL_ZPX => addressing_zeropage_x_rmw(nes, rol),
        ROL_ABS => addressing_absolute_rmw(nes, rol),
        ROL_ABSX => addressing_absolute_x_rmw(nes, rol),
        ROL_ACC => rol_acc(nes),

        ROR_ZP => addressing_zeropage_rmw(nes, ror),
        ROR_ZPX => addressing_zeropage_x_rmw(nes, ror),
        ROR_ABS => addressing_absolute_rmw(nes, ror),
        ROR_ABSX => addressing_absolute_x_rmw(nes, ror),
        ROR_ACC => ror_acc(nes),

        BIT_ZP => bit(nes, addressing_zeropage_rd),
        BIT_ABS => bit(nes, addressing_absolute_rd),

        JMP_ABS => {
            nes.PC = nes.read_code_addr();
        }
        JMP_INDIR => {
            let addr = nes.read_code_addr();
            let low = nes.read8(addr);
            // The second read wraps around within the same page of memory, eg: we read $02FF, then $0200 (instead of $0300).
            let high_addr = (addr & 0xFF00) + (addr.wrapping_add(1) & 0xFF);
            let high = nes.read8(high_addr);
            nes.PC = (high as u16) << 8 | (low as u16);
        }

        BCC_REL => branch_cond(nes, nes.SR.C == false),
        BCS_REL => branch_cond(nes, nes.SR.C == true),
        BNE_REL => branch_cond(nes, nes.SR.Z == false),
        BEQ_REL => branch_cond(nes, nes.SR.Z == true),
        BPL_REL => branch_cond(nes, nes.SR.N == false),
        BMI_REL => branch_cond(nes, nes.SR.N == true),
        BVC_REL => branch_cond(nes, nes.SR.V == false),
        BVS_REL => branch_cond(nes, nes.SR.V == true),

        BRK => nes.do_brk_interrupt(),

        PHA => {
            nes.read8(nes.PC); // Throwaway read
            nes.push8(nes.A);
        }
        PHP => {
            nes.read8(nes.PC); // Throwaway read
            let sr = nes.get_status_register() | StatusRegister::FLAG_B;
            nes.push8(sr);
        }
        PLA => {
            nes.read8(nes.PC); // Throwaway read
            nes.tick(); // Tick to decrement S
            nes.A = nes.pop8();
            update_zn(nes, nes.A);
        }
        PLP => {
            nes.read8(nes.PC); // Throwaway read
            nes.tick(); // Tick to decrement S
            pop_status_register(nes);
        }

        JSR_ABS => {
            let addr = nes.read_code_addr();
            nes.tick();
            nes.push16(nes.PC - 1);
            nes.PC = addr;
        }

        RTS => {
            nes.read8(nes.PC); // Throwaway instruction byte
            nes.tick(); // Increment S
            nes.PC = nes.pop16() + 1;
            nes.tick(); // Increment PC
        }

        RTI => {
            nes.read8(nes.PC); // Throwaway instruction byte
            nes.tick(); // Increment S
            pop_status_register(nes);
            nes.PC = nes.pop16();
        }

        // This is the only official NOP instruction
        NOP => { nes.tick(); }

        _ => {
            unimplemented_instruction(nes, op);
        }
    }
}

#[cold]
#[inline(never)]
fn log_instruction(nes: &mut NES) {
    let disassembly: String = disassemble::disassemble(nes);
    trace!("{}", disassembly);
}

#[cold]
#[inline(never)]
fn unimplemented_instruction(nes: &mut NES, op: u8) {
    unimplemented!("instruction {} (0x{op:02X}) at ${:04X}", disassemble::INSTRUCTION_NAMES[op as usize], nes.PC - 1);
}

fn pop_status_register(nes: &mut NES) {
    let new_sr = nes.pop8() & !StatusRegister::FLAG_B | StatusRegister::FLAG_U;
    nes.set_status_register(new_sr);
}

type ReadAddressing = fn(&mut NES) -> u8;
type WriteAddressing = fn(&mut NES, u8);
type RMWInstruction = fn(&mut NES, arg: u8) -> u8;

//
// Read addressing modes
//

fn addressing_immediate_rd(nes: &mut NES) -> u8 {
    nes.read_code()
}

fn addressing_zeropage_rd(nes: &mut NES) -> u8 {
    let addr = nes.read_code() as u16;
    nes.read8(addr)
}

fn addressing_zeropage_x_rd(nes: &mut NES) -> u8 {
    let base: u8 = nes.read_code();
    nes.read8(base as u16); // Thrown away read
    // The high byte of the effective address is always zero, i.e. page boundary crossings are not handled.
    let addr: u8 = base.wrapping_add(nes.X);
    nes.read8(addr as u16)
}

fn addressing_zeropage_y_rd(nes: &mut NES) -> u8 {
    let base: u8 = nes.read_code();
    nes.read8(base as u16); // Thrown away read
    // The high byte of the effective address is always zero, i.e. page boundary crossings are not handled.
    let addr: u8 = base.wrapping_add(nes.Y);
    nes.read8(addr as u16)
}

fn addressing_absolute_rd(nes: &mut NES) -> u8 {
    let addr: u16 = nes.read_code_addr();
    nes.read8(addr)
}

fn addressing_absolute_x_rd(nes: &mut NES) -> u8 {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.X as u16);
    // First we read from the effective address without handling crossing a page boundary
    let mut result: u8 = nes.read8((base & 0xFF00) | (addr & 0xFF));
    if pages_differ(base, addr) {
        // If we crossed a page boundary, re-read with the high byte carry applied.
        result = nes.read8(addr);
    }
    result
}

fn addressing_absolute_y_rd(nes: &mut NES) -> u8 {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.Y as u16);
    // First we read from the effective address without handling crossing a page boundary
    let mut result: u8 = nes.read8((base & 0xFF00) | (addr & 0xFF));
    if pages_differ(base, addr) {
        // If we crossed a page boundary, re-read with the high byte carry applied.
        result = nes.read8(addr);
    }
    result
}

fn addressing_indirect_x_rd(nes: &mut NES) -> u8 {
    let arg = nes.read_code();
    nes.read8(arg as u16); // Dummy read
    let zp_addr = arg.wrapping_add(nes.X);
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    let addr = (high as u16) << 8 | (low as u16);
    nes.read8(addr)
}

fn addressing_indirect_y_rd(nes: &mut NES) -> u8 {
    let zp_addr = nes.read_code();
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    let base = (high as u16) << 8 | (low as u16);
    let addr = base.wrapping_add(nes.Y as u16);

    // First we read from the effective address without handling crossing a page boundary
    let mut result: u8 = nes.read8((base & 0xFF00) | (addr & 0xFF));
    if pages_differ(base, addr) {
        // If we crossed a page boundary, re-read with the high byte carry applied.
        result = nes.read8(addr);
    }
    result
}

//
// Write addressing modes
//

fn addressing_zeropage_wr(nes: &mut NES, result: u8) {
    let addr = nes.read_code() as u16;
    nes.write8(addr, result);
}

fn addressing_zeropage_x_wr(nes: &mut NES, result: u8) {
    let base: u8 = nes.read_code();
    nes.read8(base as u16); // Thrown away read
    // The high byte of the effective address is always zero, i.e. page boundary crossings are not handled.
    let addr: u8 = base.wrapping_add(nes.X);
    nes.write8(addr as u16, result);
}

fn addressing_zeropage_y_wr(nes: &mut NES, result: u8) {
    let base: u8 = nes.read_code();
    nes.read8(base as u16); // Thrown away read
    // The high byte of the effective address is always zero, i.e. page boundary crossings are not handled.
    let addr: u8 = base.wrapping_add(nes.Y);
    nes.write8(addr as u16, result);
}

fn addressing_absolute_wr(nes: &mut NES, result: u8) {
    let addr: u16 = nes.read_code_addr();
    nes.write8(addr, result);
}

fn addressing_absolute_x_wr(nes: &mut NES, result: u8) {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.X as u16);
    // Dummy read before we've calculated the full effective address
    nes.read8((base & 0xFF00) | (addr & 0xFF));
    nes.write8(addr, result);
}

fn addressing_absolute_y_wr(nes: &mut NES, result: u8) {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.Y as u16);
    // Dummy read before we've calculated the full effective address
    nes.read8((base & 0xFF00) | (addr & 0xFF));
    nes.write8(addr, result);
}

fn addressing_indirect_x_wr(nes: &mut NES, result: u8) {
    let arg = nes.read_code();
    nes.read8(arg as u16); // Dummy read
    let zp_addr = arg.wrapping_add(nes.X);
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    let addr = (high as u16) << 8 | (low as u16);
    nes.write8(addr, result);
}

fn addressing_indirect_y_wr(nes: &mut NES, result: u8) {
    let zp_addr = nes.read_code();
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    let base = (high as u16) << 8 | (low as u16);
    let addr = base.wrapping_add(nes.Y as u16);
    // Dummy read before we've calculated the full effective address
    nes.read8((base & 0xFF00) | (addr & 0xFF));
    nes.write8(addr, result);
}

//
// Read-modify-write addressing modes
//

fn addressing_zeropage_rmw(nes: &mut NES, op: RMWInstruction) {
    let addr = nes.read_code() as u16;
    let arg = nes.read8(addr);
    nes.write8(addr, arg); // Redundant write-back
    let result = op(nes, arg);
    nes.write8(addr, result);
}

fn addressing_zeropage_x_rmw(nes: &mut NES, op: RMWInstruction) {
    let mut addr: u8 = nes.read_code();
    nes.read8(addr as u16); // Thrown away read
    // The high byte of the effective address is always zero, i.e. page boundary crossings are not handled.
    addr = addr.wrapping_add(nes.X);
    let arg: u8 = nes.read8(addr as u16);
    nes.write8(addr as u16, arg); // Redundant write-back
    let result: u8 = op(nes, arg);
    nes.write8(addr as u16, result);
}

fn addressing_absolute_rmw(nes: &mut NES, op: RMWInstruction) {
    let addr: u16 = nes.read_code_addr();
    let arg = nes.read8(addr);
    nes.write8(addr, arg); // Redundant write-back
    let result = op(nes, arg);
    nes.write8(addr, result);
}

fn addressing_absolute_x_rmw(nes: &mut NES, op: RMWInstruction) {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.X as u16);
    // Dummy read before we've calculated the full effective address
    nes.read8((base & 0xFF00) | (addr & 0xFF));
    let arg: u8 = nes.read8(addr);
    nes.write8(addr, arg); // Redundant write-back
    let result: u8 = op(nes, arg);
    nes.write8(addr, result);
}

//
// Instruction implementations
//

fn adc(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    adc_inner(nes, arg);
}

fn sbc(nes: &mut NES, addressing: ReadAddressing) {
    let arg = !addressing(nes);
    adc_inner(nes, arg)
}

fn adc_inner(nes: &mut NES, arg: u8) {
    let acc = nes.A;
    let sum_with_overflow = (arg as u16).wrapping_add(acc as u16).wrapping_add(nes.SR.C as u16);
    nes.A = sum_with_overflow as u8;

    nes.SR.C = (nes.A as u16) < sum_with_overflow;
    nes.SR.V = ((nes.A ^ arg) & (nes.A ^ acc) & 0x80) != 0;
    update_zn(nes, nes.A);
}

fn and(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.A &= arg;
    update_zn(nes, nes.A);
}

fn eor(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.A ^= arg;
    update_zn(nes, nes.A);
}

fn ora(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.A |= arg;
    update_zn(nes, nes.A);
}

fn cmp(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.SR.C = nes.A >= arg;
    update_zn(nes, nes.A.wrapping_sub(arg));
}

fn cpx(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.SR.C = nes.X >= arg;
    update_zn(nes, nes.X.wrapping_sub(arg));
}

fn cpy(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.SR.C = nes.Y >= arg;
    update_zn(nes, nes.Y.wrapping_sub(arg));
}

fn dec(nes: &mut NES, arg: u8) -> u8 {
    let result = arg.wrapping_sub(1);
    update_zn(nes, result);
    result
}

fn dey(nes: &mut NES) {
    nes.Y = nes.Y.wrapping_sub(1);
    update_zn(nes, nes.Y);
    nes.tick();
}

fn dex(nes: &mut NES) {
    nes.X = nes.X.wrapping_sub(1);
    update_zn(nes, nes.X);
    nes.tick();
}

fn inc(nes: &mut NES, arg: u8) -> u8 {
    let result = arg.wrapping_add(1);
    update_zn(nes, result);
    result
}

fn iny(nes: &mut NES) {
    nes.Y = nes.Y.wrapping_add(1);
    update_zn(nes, nes.Y);
    nes.tick();
}

fn inx(nes: &mut NES) {
    nes.X = nes.X.wrapping_add(1);
    update_zn(nes, nes.X);
    nes.tick();
}

fn lda(nes: &mut NES, addressing: ReadAddressing) {
    nes.A = addressing(nes);
    update_zn(nes, nes.A);
}

fn ldx(nes: &mut NES, addressing: ReadAddressing) {
    nes.X = addressing(nes);
    update_zn(nes, nes.X);
}

fn ldy(nes: &mut NES, addressing: ReadAddressing) {
    nes.Y = addressing(nes);
    update_zn(nes, nes.Y);
}

fn sta(nes: &mut NES, addressing: WriteAddressing) {
    addressing(nes, nes.A);
}

fn stx(nes: &mut NES, addressing: WriteAddressing) {
    addressing(nes, nes.X);
}

fn sty(nes: &mut NES, addressing: WriteAddressing) {
    addressing(nes, nes.Y);
}

fn lsr(nes: &mut NES, mut val: u8) -> u8 {
    nes.SR.C = val & 0x01 != 0;
    val >>= 1;
    update_zn(nes, val);
    val
}

fn lsr_acc(nes: &mut NES) {
    nes.SR.C = nes.A & 0x01 != 0;
    nes.A >>= 1;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn asl(nes: &mut NES, mut val: u8) -> u8 {
    nes.SR.C = val & 0x80 != 0;
    val <<= 1;
    update_zn(nes, val);
    val
}

fn asl_acc(nes: &mut NES) {
    nes.SR.C = nes.A & 0x80 != 0;
    nes.A <<= 1;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn rol(nes: &mut NES, mut val: u8) -> u8 {
    let new_bit_0 = nes.SR.C as u8;
    nes.SR.C = val & 0x80 != 0;
    val = (val << 1) | new_bit_0;
    update_zn(nes, val);
    val
}

fn rol_acc(nes: &mut NES) {
    let new_bit_0 = nes.SR.C as u8;
    nes.SR.C = nes.A & 0x80 != 0;
    nes.A = (nes.A << 1) | new_bit_0;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn ror(nes: &mut NES, mut val: u8) -> u8 {
    let new_bit_7 = (nes.SR.C as u8) << 7;
    nes.SR.C = val & 0x01 != 0;
    val = (val >> 1) | new_bit_7;
    update_zn(nes, val);
    val
}

fn ror_acc(nes: &mut NES) {
    let new_bit_7 = (nes.SR.C as u8) << 7;
    nes.SR.C = nes.A & 0x01 != 0;
    nes.A = (nes.A >> 1) | new_bit_7;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn bit(nes: &mut NES, addressing: ReadAddressing) {
    let arg = addressing(nes);
    nes.SR.Z = (arg & nes.A) == 0;
    nes.SR.N = arg & 0x80 != 0;
    nes.SR.V = arg & 0x40 != 0;
}

fn branch_cond(nes: &mut NES, cond: bool) {
    let offset = nes.read_code() as i8 as i16;
    if cond {
        let old_pc = nes.PC;
        nes.PC = old_pc.wrapping_add_signed(offset);
        nes.tick();
        if pages_differ(old_pc, nes.PC) {
            nes.tick();
        }
    }
}

fn update_zn(nes: &mut NES, value: u8) {
    nes.SR.Z = value == 0;
    nes.SR.N = value & 0x80 != 0;
}

fn pages_differ(addr_a: u16, addr_b: u16) -> bool {
    (addr_a & 0xFF00) != (addr_b & 0xFF00)
}

fn alu_cycle(nes: &mut NES) {
    nes.tick();
}
