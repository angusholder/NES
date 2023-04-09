use log::{trace};
use crate::nes::{NES, StatusRegister};
use crate::cpu_ops::*;
use crate::disassemble;

pub fn emulate_instruction(nes: &mut NES) {
    if nes.trace_instructions {
        let disassembly: String = disassemble::disassemble(nes);
        trace!("{}", disassembly);
    }

    let op = nes.read_code();

    match op {
        ADC_IMM => adc(nes, addressing_immediate),
        ADC_ZP => adc(nes, addressing_zeropage),
        ADC_ZPX => adc(nes, addressing_zeropage_x),
        ADC_ABS => adc(nes, addressing_absolute),
        ADC_ABSX => adc(nes, addressing_absolute_x),
        ADC_ABSY => adc(nes, addressing_absolute_y),
        ADC_INDIRX => adc(nes, addressing_indirect_x),
        ADC_INDIRY => adc(nes, addressing_indirect_y),

        SBC_IMM | 0xEB => sbc(nes, addressing_immediate),
        SBC_ZP => sbc(nes, addressing_zeropage),
        SBC_ZPX => sbc(nes, addressing_zeropage_x),
        SBC_ABS => sbc(nes, addressing_absolute),
        SBC_ABSX => sbc(nes, addressing_absolute_x),
        SBC_ABSY => sbc(nes, addressing_absolute_y),
        SBC_INDIRX => sbc(nes, addressing_indirect_x),
        SBC_INDIRY => sbc(nes, addressing_indirect_y),

        AND_IMM => and(nes, addressing_immediate),
        AND_ZP => and(nes, addressing_zeropage),
        AND_ZPX => and(nes, addressing_zeropage_x),
        AND_ABS => and(nes, addressing_absolute),
        AND_ABSX => and(nes, addressing_absolute_x),
        AND_ABSY => and(nes, addressing_absolute_y),
        AND_INDIRX => and(nes, addressing_indirect_x),
        AND_INDIRY => and(nes, addressing_indirect_y),

        EOR_IMM => eor(nes, addressing_immediate),
        EOR_ZP => eor(nes, addressing_zeropage),
        EOR_ZPX => eor(nes, addressing_zeropage_x),
        EOR_ABS => eor(nes, addressing_absolute),
        EOR_ABSX => eor(nes, addressing_absolute_x),
        EOR_ABSY => eor(nes, addressing_absolute_y),
        EOR_INDIRX => eor(nes, addressing_indirect_x),
        EOR_INDIRY => eor(nes, addressing_indirect_y),

        ORA_IMM => ora(nes, addressing_immediate),
        ORA_ZP => ora(nes, addressing_zeropage),
        ORA_ZPX => ora(nes, addressing_zeropage_x),
        ORA_ABS => ora(nes, addressing_absolute),
        ORA_ABSX => ora(nes, addressing_absolute_x),
        ORA_ABSY => ora(nes, addressing_absolute_y),
        ORA_INDIRX => ora(nes, addressing_indirect_x),
        ORA_INDIRY => ora(nes, addressing_indirect_y),

        CMP_IMM => cmp(nes, addressing_immediate),
        CMP_ZP => cmp(nes, addressing_zeropage),
        CMP_ZPX => cmp(nes, addressing_zeropage_x),
        CMP_ABS => cmp(nes, addressing_absolute),
        CMP_ABSX => cmp(nes, addressing_absolute_x),
        CMP_ABSY => cmp(nes, addressing_absolute_y),
        CMP_INDIRX => cmp(nes, addressing_indirect_x),
        CMP_INDIRY => cmp(nes, addressing_indirect_y),

        CPX_IMM => cpx(nes, addressing_immediate),
        CPX_ZP => cpx(nes, addressing_zeropage),
        CPX_ABS => cpx(nes, addressing_absolute),

        CPY_IMM => cpy(nes, addressing_immediate),
        CPY_ZP => cpy(nes, addressing_zeropage),
        CPY_ABS => cpy(nes, addressing_absolute),

        DEX => dex(nes),
        DEY => dey(nes),

        INX => inx(nes),
        INY => iny(nes),

        DEC_ZP => dec(nes, addressing_zeropage),
        DEC_ZPX => dec(nes, addressing_zeropage_x),
        DEC_ABS => dec(nes, addressing_absolute),
        DEC_ABSX => dec(nes, addressing_absolute_x),

        INC_ZP => inc(nes, addressing_zeropage),
        INC_ZPX => inc(nes, addressing_zeropage_x),
        INC_ABS => inc(nes, addressing_absolute),
        INC_ABSX => inc(nes, addressing_absolute_x),

        LDA_IMM => lda(nes, addressing_immediate),
        LDA_ZP => lda(nes, addressing_zeropage),
        LDA_ZPX => lda(nes, addressing_zeropage_x),
        LDA_ABS => lda(nes, addressing_absolute),
        LDA_ABSX => lda(nes, addressing_absolute_x),
        LDA_ABSY => lda(nes, addressing_absolute_y),
        LDA_INDIRX => lda(nes, addressing_indirect_x),
        LDA_INDIRY => lda(nes, addressing_indirect_y),

        LDX_IMM => ldx(nes, addressing_immediate),
        LDX_ZP => ldx(nes, addressing_zeropage),
        LDX_ZPY => ldx(nes, addressing_zeropage_y),
        LDX_ABS => ldx(nes, addressing_absolute),
        LDX_ABSY => ldx(nes, addressing_absolute_y),

        LDY_IMM => ldy(nes, addressing_immediate),
        LDY_ZP => ldy(nes, addressing_zeropage),
        LDY_ZPX => ldy(nes, addressing_zeropage_x),
        LDY_ABS => ldy(nes, addressing_absolute),
        LDY_ABSX => ldy(nes, addressing_absolute_x),

        STA_ZP => sta(nes, addressing_zeropage),
        STA_ZPX => sta(nes, addressing_zeropage_x),
        STA_ABS => sta(nes, addressing_absolute),
        STA_ABSX => sta(nes, addressing_absolute_x),
        STA_ABSY => sta(nes, addressing_absolute_y),
        STA_INDIRX => sta(nes, addressing_indirect_x),
        STA_INDIRY => sta(nes, addressing_indirect_y),

        STX_ZP => stx(nes, addressing_zeropage),
        STX_ZPY => stx(nes, addressing_zeropage_y),
        STX_ABS => stx(nes, addressing_absolute),

        STY_ZP => sty(nes, addressing_zeropage),
        STY_ZPX => sty(nes, addressing_zeropage_x),
        STY_ABS => sty(nes, addressing_absolute),

        TAX => { nes.X = nes.A;  update_zn(nes, nes.X) }
        TAY => { nes.Y = nes.A;  update_zn(nes, nes.Y) }
        TSX => { nes.X = nes.SP; update_zn(nes, nes.X) }
        TXA => { nes.A = nes.X;  update_zn(nes, nes.A) }
        TXS => { nes.SP = nes.X; }
        TYA => { nes.A = nes.Y;  update_zn(nes, nes.A) }

        CLC => nes.SR.C = false,
        CLD => nes.SR.D = false,
        CLI => nes.SR.I = false,
        CLV => nes.SR.V = false,
        SEC => nes.SR.C = true,
        SED => nes.SR.D = true,
        SEI => nes.SR.I = true,

        LSR_ZP => lsr(nes, addressing_zeropage),
        LSR_ZPX => lsr(nes, addressing_zeropage_x),
        LSR_ABS => lsr(nes, addressing_absolute),
        LSR_ABSX => lsr(nes, addressing_absolute_x),
        LSR_ACC => lsr_acc(nes),

        ASL_ZP => asl(nes, addressing_zeropage),
        ASL_ZPX => asl(nes, addressing_zeropage_x),
        ASL_ABS => asl(nes, addressing_absolute),
        ASL_ABSX => asl(nes, addressing_absolute_x),
        ASL_ACC => asl_acc(nes),

        ROL_ZP => rol(nes, addressing_zeropage),
        ROL_ZPX => rol(nes, addressing_zeropage_x),
        ROL_ABS => rol(nes, addressing_absolute),
        ROL_ABSX => rol(nes, addressing_absolute_x),
        ROL_ACC => rol_acc(nes),

        ROR_ZP => ror(nes, addressing_zeropage),
        ROR_ZPX => ror(nes, addressing_zeropage_x),
        ROR_ABS => ror(nes, addressing_absolute),
        ROR_ABSX => ror(nes, addressing_absolute_x),
        ROR_ACC => ror_acc(nes),

        BIT_ZP => bit(nes, addressing_zeropage),
        BIT_ABS => bit(nes, addressing_absolute),

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

        BRK => nes.interrupt(crate::nes::Interrupt::BRK),

        PHA => {
            nes.push8(nes.A);
        }
        PHP => {
            let sr = nes.get_status_register() | StatusRegister::FLAG_B;
            nes.push8(sr);
        }
        PLA => {
            nes.A = nes.pop8();
            update_zn(nes, nes.A);
        }
        PLP => {
            pop_status_register(nes);
        }

        JSR_ABS => {
            let addr = nes.read_code_addr();
            nes.push16(nes.PC - 1);
            nes.PC = addr;
        }

        RTS => {
            nes.PC = nes.pop16() + 1;
        }

        RTI => {
            pop_status_register(nes);
            nes.PC = nes.pop16();
        }

        // This is the only official NOP instruction
        NOP => {}

        _ => {
            unimplemented!("instruction {} (0x{op:02X})", disassemble::INSTRUCTION_NAMES[op as usize]);
        }
    }
}

fn pop_status_register(nes: &mut NES) {
    let new_sr = nes.pop8() & !StatusRegister::FLAG_B | StatusRegister::FLAG_U;
    nes.set_status_register(new_sr);
}

fn addressing_immediate(nes: &mut NES) -> u16 {
    let addr = nes.PC;
    nes.PC += 1;
    addr
}

fn addressing_zeropage(nes: &mut NES) -> u16 {
    nes.read_code() as u16
}

fn addressing_zeropage_x(nes: &mut NES) -> u16 {
    nes.read_code().wrapping_add(nes.X) as u16
}

fn addressing_zeropage_y(nes: &mut NES) -> u16 {
    nes.read_code().wrapping_add(nes.Y) as u16
}

fn addressing_absolute(nes: &mut NES) -> u16 {
    nes.read_code_addr()
}

fn addressing_absolute_x(nes: &mut NES) -> u16 {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.X as u16);
    if pages_differ(base, addr) {
        nes.tick();
    }
    addr
}

fn addressing_absolute_y(nes: &mut NES) -> u16 {
    let base = nes.read_code_addr();
    let addr = base.wrapping_add(nes.Y as u16);
    if pages_differ(base, addr) {
        nes.tick();
    }
    addr
}

fn addressing_indirect_x(nes: &mut NES) -> u16 {
    let zp_addr = nes.read_code().wrapping_add(nes.X);
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    (high as u16) << 8 | (low as u16)
}

fn addressing_indirect_y(nes: &mut NES) -> u16 {
    let zp_addr = nes.read_code();
    let low = nes.read8(zp_addr as u16);
    let high = nes.read8(zp_addr.wrapping_add(1) as u16);
    let base = (high as u16) << 8 | (low as u16);
    let addr = base.wrapping_add(nes.Y as u16);
    if pages_differ(base, addr) {
        nes.tick();
    }
    addr
}

fn adc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    adc_inner(nes, arg);
}

fn sbc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = !read_with(nes, addressing);
    adc_inner(nes, arg)
}

fn adc_inner(nes: &mut NES, arg: u8) {
    let acc = nes.A;
    let sum_with_overflow = (arg as u16).wrapping_add(acc as u16).wrapping_add(nes.SR.C as u16);
    nes.A = sum_with_overflow as u8;

    nes.SR.C = (nes.A as u16) < sum_with_overflow;
    nes.SR.V = ((nes.A ^ arg) & (nes.A ^ acc) & 0x80) != 0;
    update_zn(nes, nes.A);

    alu_cycle(nes);
}

fn and(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A &= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn eor(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A ^= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn ora(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A |= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn cmp(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.A >= arg;
    update_zn(nes, nes.A.wrapping_sub(arg));
    alu_cycle(nes);
}

fn cpx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.X >= arg;
    update_zn(nes, nes.X.wrapping_sub(arg));
    alu_cycle(nes);
}

fn cpy(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.Y >= arg;
    update_zn(nes, nes.Y.wrapping_sub(arg));
    alu_cycle(nes);
}

fn dec(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let arg = nes.read8(addr);
    let result = arg.wrapping_sub(1);
    nes.write8(addr, result);
    update_zn(nes, result);
    alu_cycle(nes);
}

fn dey(nes: &mut NES) {
    nes.Y = nes.Y.wrapping_sub(1);
    update_zn(nes, nes.Y);
}

fn dex(nes: &mut NES) {
    nes.X = nes.X.wrapping_sub(1);
    update_zn(nes, nes.X);
}

fn inc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let arg = nes.read8(addr);
    // Dummy write cycle of old value, relied on by some games, eg: https://www.nesdev.org/wiki/MMC1#Reset
    nes.write8(addr, arg);
    let result = arg.wrapping_add(1);
    nes.write8(addr, result);
    update_zn(nes, result);
}

fn iny(nes: &mut NES) {
    nes.Y = nes.Y.wrapping_add(1);
    update_zn(nes, nes.Y);
}

fn inx(nes: &mut NES) {
    nes.X = nes.X.wrapping_add(1);
    update_zn(nes, nes.X);
}

fn lda(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.A = read_with(nes, addressing);
    update_zn(nes, nes.A);
}

fn ldx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.X = read_with(nes, addressing);
    update_zn(nes, nes.X);
}

fn ldy(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.Y = read_with(nes, addressing);
    update_zn(nes, nes.Y);
}

fn sta(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.A);
}

fn stx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.X);
}

fn sty(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.Y);
}

fn lsr(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    nes.SR.C = val & 0x01 != 0;
    val >>= 1;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn lsr_acc(nes: &mut NES) {
    nes.SR.C = nes.A & 0x01 != 0;
    nes.A >>= 1;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn asl(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    nes.SR.C = val & 0x80 != 0;
    val <<= 1;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn asl_acc(nes: &mut NES) {
    nes.SR.C = nes.A & 0x80 != 0;
    nes.A <<= 1;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn rol(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    let new_bit_0 = nes.SR.C as u8;
    nes.SR.C = val & 0x80 != 0;
    val = (val << 1) | new_bit_0;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn rol_acc(nes: &mut NES) {
    let new_bit_0 = nes.SR.C as u8;
    nes.SR.C = nes.A & 0x80 != 0;
    nes.A = (nes.A << 1) | new_bit_0;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn ror(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    let new_bit_7 = (nes.SR.C as u8) << 7;
    nes.SR.C = val & 0x01 != 0;
    val = (val >> 1) | new_bit_7;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn ror_acc(nes: &mut NES) {
    let new_bit_7 = (nes.SR.C as u8) << 7;
    nes.SR.C = nes.A & 0x01 != 0;
    nes.A = (nes.A >> 1) | new_bit_7;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn bit(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.Z = (arg & nes.A) == 0;
    nes.SR.N = arg & 0x80 != 0;
    nes.SR.V = arg & 0x40 != 0;
    alu_cycle(nes);
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

fn read_with(nes: &mut NES, addressing: fn(&mut NES) -> u16) -> u8 {
    let addr = addressing(nes);
    nes.read8(addr)
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
