use crate::nes::{NES, StatusRegister};
use crate::ops::*;

pub fn emulate_instruction(nes: &mut NES) {
    let op = nes.read_code();

    match op {
        ADC_IMM => op_adc(nes, addressing_immediate),
        ADC_ZP => op_adc(nes, addressing_zeropage),
        ADC_ZPX => op_adc(nes, addressing_zeropage_x),
        ADC_ABS => op_adc(nes, addressing_absolute),
        ADC_ABSX => op_adc(nes, addressing_absolute_x),
        ADC_ABSY => op_adc(nes, addressing_absolute_y),
        ADC_INDIRX => op_adc(nes, addressing_indirect_x),
        ADC_INDIRY => op_adc(nes, addressing_indirect_y),

        SBC_IMM => op_sbc(nes, addressing_immediate),
        SBC_ZP => op_sbc(nes, addressing_zeropage),
        SBC_ZPX => op_sbc(nes, addressing_zeropage_x),
        SBC_ABS => op_sbc(nes, addressing_absolute),
        SBC_ABSX => op_sbc(nes, addressing_absolute_x),
        SBC_ABSY => op_sbc(nes, addressing_absolute_y),
        SBC_INDIRX => op_sbc(nes, addressing_indirect_x),
        SBC_INDIRY => op_sbc(nes, addressing_indirect_y),

        AND_IMM => op_and(nes, addressing_immediate),
        AND_ZP => op_and(nes, addressing_zeropage),
        AND_ZPX => op_and(nes, addressing_zeropage_x),
        AND_ABS => op_and(nes, addressing_absolute),
        AND_ABSX => op_and(nes, addressing_absolute_x),
        AND_ABSY => op_and(nes, addressing_absolute_y),
        AND_INDIRX => op_and(nes, addressing_indirect_x),
        AND_INDIRY => op_and(nes, addressing_indirect_y),

        EOR_IMM => op_eor(nes, addressing_immediate),
        EOR_ZP => op_eor(nes, addressing_zeropage),
        EOR_ZPX => op_eor(nes, addressing_zeropage_x),
        EOR_ABS => op_eor(nes, addressing_absolute),
        EOR_ABSX => op_eor(nes, addressing_absolute_x),
        EOR_ABSY => op_eor(nes, addressing_absolute_y),
        EOR_INDIRX => op_eor(nes, addressing_indirect_x),
        EOR_INDIRY => op_eor(nes, addressing_indirect_y),

        ORA_IMM => op_ora(nes, addressing_immediate),
        ORA_ZP => op_ora(nes, addressing_zeropage),
        ORA_ZPX => op_ora(nes, addressing_zeropage_x),
        ORA_ABS => op_ora(nes, addressing_absolute),
        ORA_ABSX => op_ora(nes, addressing_absolute_x),
        ORA_ABSY => op_ora(nes, addressing_absolute_y),
        ORA_INDIRX => op_ora(nes, addressing_indirect_x),
        ORA_INDIRY => op_ora(nes, addressing_indirect_y),

        CMP_IMM => op_cmp(nes, addressing_immediate),
        CMP_ZP => op_cmp(nes, addressing_zeropage),
        CMP_ZPX => op_cmp(nes, addressing_zeropage_x),
        CMP_ABS => op_cmp(nes, addressing_absolute),
        CMP_ABSX => op_cmp(nes, addressing_absolute_x),
        CMP_ABSY => op_cmp(nes, addressing_absolute_y),
        CMP_INDIRX => op_cmp(nes, addressing_indirect_x),
        CMP_INDIRY => op_cmp(nes, addressing_indirect_y),

        CPX_IMM => op_cpx(nes, addressing_immediate),
        CPX_ZP => op_cpx(nes, addressing_zeropage),
        CPX_ABS => op_cpx(nes, addressing_absolute),

        CPY_IMM => op_cpy(nes, addressing_immediate),
        CPY_ZP => op_cpy(nes, addressing_zeropage),
        CPY_ABS => op_cpy(nes, addressing_absolute),

        DEX_IMPLIED => {
            nes.X = nes.X.wrapping_sub(1);
            update_zn(nes, nes.X);
        }

        DEY_IMPLIED => {
            nes.Y = nes.Y.wrapping_sub(1);
            update_zn(nes, nes.Y);
        }

        INX_IMPLIED => {
            nes.X = nes.X.wrapping_add(1);
            update_zn(nes, nes.X);
        }

        INY_IMPLIED => {
            nes.Y = nes.Y.wrapping_add(1);
            update_zn(nes, nes.Y);
        }

        DEC_ZP => op_dec(nes, addressing_zeropage),
        DEC_ZPX => op_dec(nes, addressing_zeropage_x),
        DEC_ABS => op_dec(nes, addressing_absolute),
        DEC_ABSX => op_dec(nes, addressing_absolute_x),

        INC_ZP => op_inc(nes, addressing_zeropage),
        INC_ZPX => op_inc(nes, addressing_zeropage_x),
        INC_ABS => op_inc(nes, addressing_absolute),
        INC_ABSX => op_inc(nes, addressing_absolute_x),

        LDA_IMM => op_lda(nes, addressing_immediate),
        LDA_ZP => op_lda(nes, addressing_zeropage),
        LDA_ZPX => op_lda(nes, addressing_zeropage_x),
        LDA_ABS => op_lda(nes, addressing_absolute),
        LDA_ABSX => op_lda(nes, addressing_absolute_x),
        LDA_ABSY => op_lda(nes, addressing_absolute_y),
        LDA_INDIRX => op_lda(nes, addressing_indirect_x),
        LDA_INDIRY => op_lda(nes, addressing_indirect_y),

        LDX_IMM => op_ldx(nes, addressing_immediate),
        LDX_ZP => op_ldx(nes, addressing_zeropage),
        LDX_ZPY => op_ldx(nes, addressing_zeropage_y),
        LDX_ABS => op_ldx(nes, addressing_absolute),
        LDX_ABSY => op_ldx(nes, addressing_absolute_y),

        LDY_IMM => op_ldy(nes, addressing_immediate),
        LDY_ZP => op_ldy(nes, addressing_zeropage),
        LDY_ZPX => op_ldy(nes, addressing_zeropage_x),
        LDY_ABS => op_ldy(nes, addressing_absolute),
        LDY_ABSX => op_ldy(nes, addressing_absolute_x),

        STA_ZP => op_sta(nes, addressing_zeropage),
        STA_ZPX => op_sta(nes, addressing_zeropage_x),
        STA_ABS => op_sta(nes, addressing_absolute),
        STA_ABSX => op_sta(nes, addressing_absolute_x),
        STA_ABSY => op_sta(nes, addressing_absolute_y),
        STA_INDIRX => op_sta(nes, addressing_indirect_x),
        STA_INDIRY => op_sta(nes, addressing_indirect_y),

        STX_ZP => op_stx(nes, addressing_zeropage),
        STX_ZPY => op_stx(nes, addressing_zeropage_y),
        STX_ABS => op_stx(nes, addressing_absolute),

        STY_ZP => op_sty(nes, addressing_zeropage),
        STY_ZPX => op_sty(nes, addressing_zeropage_x),
        STY_ABS => op_sty(nes, addressing_absolute),

        TAX_IMPLIED => { nes.X = nes.A;  update_zn(nes, nes.X) }
        TAY_IMPLIED => { nes.Y = nes.A;  update_zn(nes, nes.Y) }
        TSX_IMPLIED => { nes.X = nes.SP; update_zn(nes, nes.X) }
        TXA_IMPLIED => { nes.A = nes.X;  update_zn(nes, nes.A) }
        TXS_IMPLIED => { nes.SP = nes.X; }
        TYA_IMPLIED => { nes.A = nes.Y;  update_zn(nes, nes.A) }

        CLC_IMPLIED => nes.SR.C = false,
        CLD_IMPLIED => nes.SR.D = false,
        CLI_IMPLIED => nes.SR.I = false,
        CLV_IMPLIED => nes.SR.V = false,
        SEC_IMPLIED => nes.SR.C = true,
        SED_IMPLIED => nes.SR.D = true,
        SEI_IMPLIED => nes.SR.I = true,

        LSR_ZP => op_lsr(nes, addressing_zeropage),
        LSR_ZPX => op_lsr(nes, addressing_zeropage_x),
        LSR_ABS => op_lsr(nes, addressing_absolute),
        LSR_ABSX => op_lsr(nes, addressing_absolute_x),
        LSR_ACC => {
            nes.SR.C = nes.A & 0x01 != 0;
            nes.A >>= 1;
            update_zn(nes, nes.A);
            alu_cycle(nes);
        }

        ASL_ZP => op_asl(nes, addressing_zeropage),
        ASL_ZPX => op_asl(nes, addressing_zeropage_x),
        ASL_ABS => op_asl(nes, addressing_absolute),
        ASL_ABSX => op_asl(nes, addressing_absolute_x),
        ASL_ACC => {
            nes.SR.C = nes.A & 0x80 != 0;
            nes.A <<= 1;
            update_zn(nes, nes.A);
            alu_cycle(nes);
        }

        ROL_ZP => op_rol(nes, addressing_zeropage),
        ROL_ZPX => op_rol(nes, addressing_zeropage_x),
        ROL_ABS => op_rol(nes, addressing_absolute),
        ROL_ABSX => op_rol(nes, addressing_absolute_x),
        ROL_ACC => {
            let new_bit_0 = nes.SR.C as u8;
            nes.SR.C = nes.A & 0x80 != 0;
            nes.A = (nes.A << 1) | new_bit_0;
            update_zn(nes, nes.A);
            alu_cycle(nes);
        }

        ROR_ZP => op_ror(nes, addressing_zeropage),
        ROR_ZPX => op_ror(nes, addressing_zeropage_x),
        ROR_ABS => op_ror(nes, addressing_absolute),
        ROR_ABSX => op_ror(nes, addressing_absolute_x),
        ROR_ACC => {
            let new_bit_7 = (nes.SR.C as u8) << 7;
            nes.SR.C = nes.A & 0x01 != 0;
            nes.A = (nes.A >> 1) | new_bit_7;
            update_zn(nes, nes.A);
            alu_cycle(nes);
        }

        BIT_ZP => op_bit(nes, addressing_zeropage),
        BIT_ABS => op_bit(nes, addressing_absolute),

        JMP_ABS => {
            nes.PC = nes.read_code_addr();
        }
        JMP_INDIR => {
            let addr = nes.read_code_addr();
            nes.PC = nes.read_addr(addr);
        }

        BCC_REL => op_branch_cond(nes, nes.SR.C == false),
        BCS_REL => op_branch_cond(nes, nes.SR.C == true),
        BNE_REL => op_branch_cond(nes, nes.SR.Z == false),
        BEQ_REL => op_branch_cond(nes, nes.SR.Z == true),
        BPL_REL => op_branch_cond(nes, nes.SR.N == false),
        BMI_REL => op_branch_cond(nes, nes.SR.N == true),
        BVC_REL => op_branch_cond(nes, nes.SR.V == false),
        BVS_REL => op_branch_cond(nes, nes.SR.V == true),

        BRK_IMPLIED => nes.interrupt(crate::nes::Interrupt::BRK),

        PHA_IMPLIED => {
            nes.push8(nes.A);
        }
        PHP_IMPLIED => {
            let sr = nes.get_status_register() | StatusRegister::FLAG_B;
            nes.push8(sr);
        }
        PLA_IMPLIED => {
            nes.A = nes.pop8();
            update_zn(nes, nes.A);
        }
        PLP_IMPLIED => {
            pop_status_register(nes);
        }

        JSR_ABS => {
            let addr = nes.read_code_addr();
            nes.push16(nes.PC - 1);
            nes.PC = addr;
        }

        RTS_IMPLIED => {
            nes.PC = nes.pop16() + 1;
        }

        RTI_IMPLIED => {
            pop_status_register(nes);
            nes.PC = nes.pop16();
        }

        // This is the only official NOP instruction
        NOP_24 => {}

        _ => {
            unimplemented!("Unimplemented opcode: {:02X}", op)
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
    let addr = nes.read_addr(zp_addr as u16);
    addr
}

fn addressing_indirect_y(nes: &mut NES) -> u16 {
    let zp_addr = nes.read_code();
    let base = nes.read_addr(zp_addr as u16);
    let addr = base.wrapping_add(nes.Y as u16);
    if pages_differ(base, addr) {
        nes.tick();
    }
    addr
}

fn op_adc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    adc_inner(nes, arg);
}

fn op_sbc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
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

fn op_and(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A &= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn op_eor(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A ^= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn op_ora(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.A |= arg;
    update_zn(nes, nes.A);
    alu_cycle(nes);
}

fn op_cmp(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.A >= arg;
    update_zn(nes, nes.A.wrapping_sub(arg));
    alu_cycle(nes);
}

fn op_cpx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.X >= arg;
    update_zn(nes, nes.X.wrapping_sub(arg));
    alu_cycle(nes);
}

fn op_cpy(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.C = nes.Y >= arg;
    update_zn(nes, nes.Y.wrapping_sub(arg));
    alu_cycle(nes);
}

fn op_dec(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let arg = nes.read8(addr);
    let result = arg.wrapping_sub(1);
    nes.write8(addr, result);
    update_zn(nes, result);
    alu_cycle(nes);
}

fn op_inc(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let arg = nes.read8(addr);
    let result = arg.wrapping_add(1);
    nes.write8(addr, result);
    update_zn(nes, result);
    alu_cycle(nes);
}

fn op_lda(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.A = read_with(nes, addressing);
    update_zn(nes, nes.A);
}

fn op_ldx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.X = read_with(nes, addressing);
    update_zn(nes, nes.X);
}

fn op_ldy(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    nes.Y = read_with(nes, addressing);
    update_zn(nes, nes.Y);
}

fn op_sta(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.A);
}

fn op_stx(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.X);
}

fn op_sty(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    nes.write8(addr, nes.Y);
}

fn op_lsr(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    nes.SR.C = val & 0x01 != 0;
    val >>= 1;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn op_asl(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    nes.SR.C = val & 0x80 != 0;
    val <<= 1;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn op_rol(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    let new_bit_0 = nes.SR.C as u8;
    nes.SR.C = val & 0x80 != 0;
    val = (val << 1) | new_bit_0;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn op_ror(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let addr = addressing(nes);
    let mut val = nes.read8(addr);
    let new_bit_7 = (nes.SR.C as u8) << 7;
    nes.SR.C = val & 0x01 != 0;
    val = (val >> 1) | new_bit_7;
    nes.write8(addr, val);
    update_zn(nes, val);
    alu_cycle(nes);
}

fn op_bit(nes: &mut NES, addressing: fn(&mut NES) -> u16) {
    let arg = read_with(nes, addressing);
    nes.SR.Z = (arg & nes.A) == 0;
    nes.SR.N = arg & 0x80 != 0;
    nes.SR.V = arg & 0x40 != 0;
    alu_cycle(nes);
}

fn op_branch_cond(nes: &mut NES, cond: bool) {
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
