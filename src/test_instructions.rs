use crate::mapper::Mapper;
use crate::ops::*;
use crate::nes::{ NES, StatusRegister };
use crate::ppu::PPU;

fn new_nes() -> NES {
    let mapper: Mapper = Mapper::new(crate::cartridge::Cartridge {
        prg_rom: vec![0; 0x4000],
        chr_rom: vec![0; 0x2000],
        mapper_num: 0,
        mirroring: crate::cartridge::Mirror::Horizontal,
    }).unwrap();

    let mut nes = NES::new(mapper);
    nes.A = 0;
    nes.X = 0;
    nes.Y = 0;
    nes.SP = 0xFD;
    nes.PC = 0;
    nes.SR = StatusRegister::from_byte(0);

    nes
}

fn push8(nes: &mut NES, value: u8) {
    nes.push8(value);
}

fn pop8(nes: &mut NES) -> u8 {
    nes.pop8()
}

fn emulate_instructions(nes: &mut NES, num: usize) {
    for _ in 0..num {
        crate::instructions::emulate_instruction(nes);
    }
}

mod bpl_implied {
    use super::*;

    #[test]
    fn with_neg() {
        let mut nes = &mut new_nes();
        nes.SR.N = true;
        nes.SR.D = false;
        nes.ram[0] = 0x10;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }

    #[test]
    fn no_neg() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.D = false;
        nes.ram[0] = 0x10;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }
}

mod lda_immediate {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa9;
        nes.ram[1] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa9;
        nes.ram[1] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa9;
        nes.ram[1] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_zeropage {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_absolute {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xad;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xad;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xad;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod lda_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldx_immediate {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa2;
        nes.ram[1] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa2;
        nes.ram[1] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa2;
        nes.ram[1] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldx_zeropage {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldx_zeropage_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldx_absolute {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xae;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xae;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xae;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldx_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbe;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbe;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbe;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.X);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldy_immediate {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa0;
        nes.ram[1] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa0;
        nes.ram[1] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa0;
        nes.ram[1] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldy_zeropage {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xa4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldy_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb4;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb4;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xb4;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldy_absolute {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xac;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xac;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xac;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ldy_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbc;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbc;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0x20, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.Z = false;
        nes.ram[0] = 0xbc;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.Y);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod clc_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x18;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.C);
    }
}

mod cld_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.D = true;
        nes.ram[0] = 0xd8;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.D);
    }
}

mod cli_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.I = true;
        nes.ram[0] = 0x58;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.I);
    }
}

mod clv_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.V = true;
        nes.ram[0] = 0xb8;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.V);
    }
}

mod cpx_immediate {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xe0;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xe0;
        nes.ram[1] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x04;
        nes.ram[0] = 0xe0;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cpx_zeropage {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xe4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xe4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x04;
        nes.ram[0] = 0xe4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cpx_absolute {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xec;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x10;
        nes.ram[0] = 0xec;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.X = 0x04;
        nes.ram[0] = 0xec;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod stx_zeropage {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x23;
        nes.ram[0] = 0x86;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod stx_zeropage_y {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.X = 0x23;
        nes.ram[0] = 0x96;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod stx_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x23;
        nes.ram[0] = 0x8e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[3]);
    }
}

mod sty_zeropage {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x23;
        nes.ram[0] = 0x84;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod sty_zeropage_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x23;
        nes.ram[0] = 0x94;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod sty_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x23;
        nes.ram[0] = 0x8c;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[3]);
    }
}

mod pha_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        let begin_sp = nes.SP;
        nes.A = 0x23;
        nes.ram[0] = 0x48;
        emulate_instructions(nes, 1);
        assert_eq!(1, begin_sp - nes.SP);
        assert_eq!(0x23, pop8(nes));
    }
}

mod inc_zeropage {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xe6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xe6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod inc_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xf6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xf6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod inc_absolute {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xee;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xee;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod inc_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xfe;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xfe;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod inx_implied {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.X = 0xfe;
        nes.ram[0] = 0xe8;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.X);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.X = 0xff;
        nes.ram[0] = 0xe8;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod iny_implied {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.Y = 0xfe;
        nes.ram[0] = 0xc8;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.Y);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.Y = 0xff;
        nes.ram[0] = 0xc8;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod cpy_immediate {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xc0;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xc0;
        nes.ram[1] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x04;
        nes.ram[0] = 0xc0;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cpy_zeropage {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xc4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xc4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x04;
        nes.ram[0] = 0xc4;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cpy_absolute {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xcc;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x10;
        nes.ram[0] = 0xcc;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.Y = 0x04;
        nes.ram[0] = 0xcc;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod eor_immediate {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0xce;
        nes.ram[0] = 0x49;
        nes.ram[1] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_zeropage {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0xce;
        nes.ram[0] = 0x45;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_zeropage_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0xce;
        nes.ram[0] = 0x55;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0xce;
        nes.ram[0] = 0x4d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_absolute_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0xce;
        nes.ram[0] = 0x5d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_absolute_y {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.X = 0x00;
        nes.A = 0xce;
        nes.ram[0] = 0x59;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_indirect_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0xce;
        nes.ram[0] = 0x41;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod eor_indirect_y {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.X = 0x00;
        nes.A = 0xce;
        nes.ram[0] = 0x51;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x20;
        emulate_instructions(nes, 1);
        assert_eq!(0xce ^ 0x20, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
    }
}

mod ror_implied {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.A = 0x81;
        nes.SR.C = true;
        nes.ram[0] = 0x6a;
        emulate_instructions(nes, 1);
        assert_eq!(0xc0, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.A = 0x01;
        nes.SR.C = false;
        nes.ram[0] = 0x6a;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod ror_zeropage {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x66;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0xc0, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x66;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod ror_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x76;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0xc0, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x76;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod ror_absolute {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x6e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0xc0, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x6e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod ror_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x7e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0xc0, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x7e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod and_immediate {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x29;
        nes.ram[1] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x29;
        nes.ram[1] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_zeropage {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x25;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x25;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;
    nes.A = 0xff;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x35;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x35;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_absolute {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x2d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x2d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;
    nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x3d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x3d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;
    nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x39;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x39;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;
    nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x21;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x21;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod and_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.A = 0xff;
    nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn all_high() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x31;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn all_low() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x31;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod dec_zeropage {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xc6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xc6;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod dec_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xd6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xd6;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod dec_absolute {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xce;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xce;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod dec_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xde;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0xde;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod dex_implied {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.ram[0] = 0xca;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.X);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.X = 0x00;
        nes.ram[0] = 0xca;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.X);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod dey_implied {
    use super::*;

    #[test]
    fn no_wrap() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.ram[0] = 0x88;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.Y);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }

    #[test]
    fn wrap() {
        let mut nes = &mut new_nes();
        nes.X = 0x00;
        nes.ram[0] = 0x88;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.Y);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod bvc_implied {
    use super::*;

    #[test]
    fn with_over() {
        let mut nes = &mut new_nes();
        nes.SR.V = true;
        nes.SR.D = false;
        nes.ram[0] = 0x50;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }

    #[test]
    fn no_over() {
        let mut nes = &mut new_nes();
        nes.SR.V = false;
        nes.SR.D = false;
        nes.ram[0] = 0x50;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }
}

mod rol_implied {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.A = 0x81;
        nes.SR.C = true;
        nes.ram[0] = 0x2a;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x2a;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod rol_zeropage {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x26;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x26;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod rol_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x36;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x36;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod rol_absolute {
    use super::*;

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x2e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x2e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod rol_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn rotate_in() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x3e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x81;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn rotate_out() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x3e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.C);
    }
}

mod bmi_implied {
    use super::*;

    #[test]
    fn with_neg() {
        let mut nes = &mut new_nes();
        nes.SR.N = true;
        nes.SR.D = false;
        nes.ram[0] = 0x30;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }

    #[test]
    fn no_neg() {
        let mut nes = &mut new_nes();
        nes.SR.N = false;
        nes.SR.D = false;
        nes.ram[0] = 0x30;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }
}

mod plp_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.set_status_register(0x20);
        push8(nes, 0xBA);
        nes.ram[0] = 0x28;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.V);
        assert_eq!(true, nes.SR.D);
        assert_eq!(false, nes.SR.I);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod jmp_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.D = false;
        nes.ram[0] = JMP_ABS;
        nes.ram[1] = 0x04;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        nes.ram[4] = SED_IMPLIED;
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }
}

mod jmp_indirect {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.D = false;
        nes.ram[0] = JMP_INDIR;
        nes.ram[1] = 0x04;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x06;
        nes.ram[5] = 0x00;
        nes.ram[6] = SED_IMPLIED;
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }
}

mod jsr_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.D = false;
        nes.ram[0] = 0x20;
        nes.ram[1] = 0x04;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
        assert_eq!(0x02, nes.pop16())// 1 less than PC is pushed by jsr
    }
}

mod bvs_implied {
    use super::*;

    #[test]
    fn with_over() {
        let mut nes = &mut new_nes();
        nes.SR.V = true;
        nes.SR.D = false;
        nes.ram[0] = 0x70;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }

    #[test]
    fn no_over() {
        let mut nes = &mut new_nes();
        nes.SR.V = false;
        nes.SR.D = false;
        nes.ram[0] = 0x70;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }
}

mod tax_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0;
        nes.A = 0x23;
        nes.ram[0] = 0xaa;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.X);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.X = 0;
        nes.A = 0xfe;
        nes.ram[0] = 0xaa;
        emulate_instructions(nes, 1);
        assert_eq!(0xfe, nes.X);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod tay_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0;
        nes.A = 0x23;
        nes.ram[0] = 0xa8;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.Y);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.Y = 0;
        nes.A = 0xfe;
        nes.ram[0] = 0xa8;
        emulate_instructions(nes, 1);
        assert_eq!(0xfe, nes.Y);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod tsx_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0;
        nes.SP = 0x23;
        nes.ram[0] = 0xba;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.X);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.X = 0;
        nes.SP = 0xfe;
        nes.ram[0] = 0xba;
        emulate_instructions(nes, 1);
        assert_eq!(0xfe, nes.X);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod txa_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0;
        nes.X = 0x23;
        nes.ram[0] = 0x8a;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.A = 0;
        nes.X = 0xfe;
        nes.ram[0] = 0x8a;
        emulate_instructions(nes, 1);
        assert_eq!(0xfe, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod txs_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.set_status_register(0x20); // aka SR_FIXED_BITS
        nes.X = 0x23;
        nes.SP = 0x00;
        nes.ram[0] = 0x9a;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.SP);
        assert_eq!(false, nes.SR.Z);
    }
}

mod tya_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0;
        nes.Y = 0x23;
        nes.ram[0] = 0x98;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn negative() {
        let mut nes = &mut new_nes();
        nes.A = 0;
        nes.Y = 0xfe;
        nes.ram[0] = 0x98;
        emulate_instructions(nes, 1);
        assert_eq!(0xfe, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }
}

mod beq_implied {
    use super::*;

    #[test]
    fn with_zero() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.D = false;
        nes.ram[0] = 0xf0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_ne!(false, nes.SR.D);
    }

    #[test]
    fn no_zero() {
        let mut nes = &mut new_nes();
        nes.SR.Z = false;
        nes.SR.D = false;
        nes.ram[0] = 0xf0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }
}

mod pla_implied {
    use super::*;

    #[test]
    fn no_flags() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        push8(nes, 0x23);
        nes.ram[0] = 0x68;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn zero_flag() {
        let mut nes = &mut new_nes();
        nes.A = 0x23;
        push8(nes, 0x00);
        nes.ram[0] = 0x68;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.N);
    }
}

mod bcc_implied {
    use super::*;

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.SR.D = false;
        nes.ram[0] = 0x90;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.SR.D = false;
        nes.ram[0] = 0x90;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_ne!(false, nes.SR.D);
    }
}

mod bcs_implied {
    use super::*;

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.SR.D = false;
        nes.ram[0] = 0xb0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_ne!(false, nes.SR.D);
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.SR.D = false;
        nes.ram[0] = 0xb0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }
}

mod rts_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        push8(nes, 0x00); // high byte pc
        push8(nes, 0x02); // low byte pc
        nes.ram[0] = 0x60;
        emulate_instructions(nes, 1);
        assert_eq!(0x03, nes.PC);
    }
}

mod rti_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        push8(nes, 0x00); // high byte pc
        push8(nes, 0x02); // low byte pc
        push8(nes, 0xba); // SR
        nes.ram[0] = 0x40;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.V);
        assert_eq!(true, nes.SR.D);
        assert_eq!(false, nes.SR.I);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(0x02, nes.PC);
    }
}

mod adc_immediate {
    use super::*;

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x69;
        nes.ram[1] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_zeropage {
    use super::*;

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = ADC_ZP;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x75;
        nes.ram[1] = 0x02;
        nes.ram[3] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_absolute {
    use super::*;

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x6d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x7d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x79;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01; // +1
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x61;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xf0; // +70
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod adc_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = false;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x01, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn with_carry() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.SR.C = true;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn negative_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xfe;
        nes.SR.C = false;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xfe;
        emulate_instructions(nes, 1);
        assert_eq!(0xfc, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow_result() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.SR.C = false;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn zero_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xff; // -1
        nes.SR.C = false;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_result() {
        let mut nes = &mut new_nes();
        nes.A = 0xf0; // +70
        nes.SR.C = false;
        nes.ram[0] = 0x71;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xf0;
        emulate_instructions(nes, 1);
        assert_eq!(0xe0, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod sec_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.ram[0] = 0x38;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.C);
    }
}

mod sed_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.D = false;
        nes.ram[0] = 0xf8;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.D);
    }
}

mod sei_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.I = false;
        nes.ram[0] = 0x78;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.I);
    }
}

mod bne_implied {
    use super::*;

    #[test]
    fn no_zero() {
        let mut nes = &mut new_nes();
        nes.SR.Z = false;
        nes.SR.D = false;
        nes.ram[0] = 0xd0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8; // SED
        emulate_instructions(nes, 2);
        assert_eq!(true, nes.SR.D);
    }

    #[test]
    fn with_zero() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.D = false;
        nes.ram[0] = 0xd0;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xf8;
        emulate_instructions(nes, 2);
        assert_eq!(false, nes.SR.D);
    }
}

mod ora_immediate {
    use super::*;

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x09;
        nes.ram[1] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x09;
        nes.ram[1] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_zeropage {
    use super::*;

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x05;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x05;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x15;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x15;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_absolute {
    use super::*;

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x0d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x0d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x1d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x1d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x00;
    nes.Y = 0x01;

        nes
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x19;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x19;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x01;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x01;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod ora_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.A = 0xff;
        nes.ram[0] = 0x11;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x0f;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x11;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x02;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
    }
}

mod lsr_implied {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.SR.D = false;
    nes.SR.N = false;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x01;
        nes.ram[0] = 0x4a;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn carry_out() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.ram[0] = 0x4a;
        emulate_instructions(nes, 1);
        assert_eq!(0x40, nes.A);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }
}

mod lsr_zeropage {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x46;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn carry_out() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x46;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x40, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }
}

mod lsr_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x56;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn carry_out() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x56;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x40, nes.ram[2]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }
}

mod lsr_absolute {
    use super::*;

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x4e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn carry_out() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x4e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x40, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }
}

mod lsr_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn zero() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.ram[0] = 0x5e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }

    #[test]
    fn carry_out() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x5e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x40, nes.ram[3]);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.N);
    }
}

mod asl_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0x01;
        nes.ram[0] = 0x0a;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_zero() {
        let mut nes = &mut new_nes();
        nes.A = 0x80;
        nes.ram[0] = 0x0a;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_negative() {
        let mut nes = &mut new_nes();
        nes.A = 0xc0;
        nes.ram[0] = 0x0a;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod asl_zeropage {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x06;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_zero() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x06;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_negative() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x06;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xc0;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod asl_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x16;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_zero() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x16;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[2]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_negative() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x16;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0xc0;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.ram[2]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod asl_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x0e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_zero() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x0e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_negative() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x0e;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xc0;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod asl_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x1e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0x02, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_zero() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x1e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x80;
        emulate_instructions(nes, 1);
        assert_eq!(0x00, nes.ram[3]);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn carry_and_negative() {
        let mut nes = &mut new_nes();
        nes.ram[0] = 0x1e;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xc0;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.ram[3]);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }
}

mod bit_zeropage {
    use super::*;

    #[test]
    fn bits_set() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x24;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn bits_unset() {
        let mut nes = &mut new_nes();
        nes.A = 0x01;
        nes.ram[0] = 0x24;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.V);
    }
}

mod bit_absolute {
    use super::*;

    #[test]
    fn bits_set() {
        let mut nes = &mut new_nes();
        nes.A = 0x00;
        nes.ram[0] = 0x24;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        // assert_eq!(false, nes.SR.C);
        // Carry flag is unaffected by Bit op, why was this tested?
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn bits_unset() {
        let mut nes = &mut new_nes();
        nes.A = 0x01;
        nes.ram[0] = 0x24;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        // assert_eq!(false, nes.SR.C);
        // Carry flag is unaffected by Bit op, why was this tested?
        assert_eq!(false, nes.SR.V);
    }
}

mod cmp_immediate {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc9;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc9;
        nes.ram[1] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xc9;
        nes.ram[1] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_zeropage {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xc5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xd5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_absolute {
    use super::*;

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xcd;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xcd;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xcd;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xdd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xdd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xdd;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xd9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xc1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xc1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod cmp_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_difference() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(true, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn less() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x10;
        nes.ram[0] = 0xd1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x04;
        emulate_instructions(nes, 1);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
    }

    #[test]
    fn more() {
        let mut nes = &mut new_nes();
        nes.SR.Z = true;
        nes.SR.C = true;
        nes.SR.N = true;
        nes.A = 0x04;
        nes.ram[0] = 0xd1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x10;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
    }
}

mod brk_implied {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.SR.I = false;
        nes.SP = 0xff;
        nes.ram[0] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(true, nes.SR.I);
        assert_eq!(0xfc, nes.SP);
        assert_eq!(nes.read_addr(0xfffe), nes.PC);
    }
}

mod sbc_immediate {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.SR.D = false;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xe9;
        nes.ram[1] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xe9;
        nes.ram[1] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xe9;
        nes.ram[1] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_zeropage {
    use super::*;

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xe5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xe5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xe5;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_zeropage_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xf5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xf5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xf5;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_absolute {
    use super::*;

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xed;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xed;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xed;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_absolute_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = SBC_ABSX;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = SBC_ABSX;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = SBC_ABSX;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_absolute_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xf9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xf9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xf9;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_indirect_x {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.X = 0x01;
    nes.Y = 0x00;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xe1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xe1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xe1;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sbc_indirect_y {
    use super::*;
    fn new_nes() -> NES {
        let mut nes = super::new_nes();
        nes.Y = 0x01;
    nes.X = 0x00;

        nes
    }

    #[test]
    fn no_overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x00;
        nes.ram[0] = 0xf1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x01;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(false, nes.SR.V);
    }

    #[test]
    fn overflow() {
        let mut nes = &mut new_nes();
        nes.SR.C = true;
        nes.A = 0x7f;
        nes.ram[0] = 0xf1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0xff;
        emulate_instructions(nes, 1);
        assert_eq!(0x80, nes.A);
        assert_eq!(true, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(false, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }

    #[test]
    fn no_borrow() {
        let mut nes = &mut new_nes();
        nes.SR.C = false;
        nes.A = 0xc0;
        nes.ram[0] = 0xf1;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x40;
        emulate_instructions(nes, 1);
        // overflow differs from 2nd sim ...
        assert_eq!(0x7f, nes.A);
        assert_eq!(false, nes.SR.N);
        assert_eq!(false, nes.SR.Z);
        assert_eq!(true, nes.SR.C);
        assert_eq!(true, nes.SR.V);
    }
}

mod sta_zeropage {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0x23;
        nes.ram[0] = 0x85;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod sta_zeropage_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0x23;
        nes.ram[0] = 0x95;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[2]);
    }
}

mod sta_absolute {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.A = 0x23;
        nes.ram[0] = 0x8d;
        nes.ram[1] = 0x03;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[3]);
    }
}

mod sta_absolute_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0x23;
        nes.ram[0] = 0x9d;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[3]);
    }
}

mod sta_absolute_y {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.X = 0x00;
        nes.A = 0x23;
        nes.ram[0] = 0x99;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x00;
        nes.ram[3] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[3]);
    }
}

mod sta_indirect_x {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.X = 0x01;
        nes.Y = 0x00;
        nes.A = 0x23;
        nes.ram[0] = 0x81;
        nes.ram[1] = 0x01;
        nes.ram[2] = 0x04;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[4]);
    }
}

mod sta_indirect_y {
    use super::*;

    #[test]
    fn normal() {
        let mut nes = &mut new_nes();
        nes.Y = 0x01;
        nes.X = 0x00;
        nes.A = 0x23;
        nes.ram[0] = 0x91;
        nes.ram[1] = 0x02;
        nes.ram[2] = 0x03;
        nes.ram[3] = 0x00;
        nes.ram[4] = 0x00;
        emulate_instructions(nes, 1);
        assert_eq!(0x23, nes.ram[4]);
    }
}

mod php_implied {
    use super::*;

    #[test]
    fn random_pattern() {
        let mut nes = &mut new_nes();
        nes.SR.N = true; nes.SR.V = false; nes.SR.D = true; nes.SR.I = false; nes.SR.Z = true; nes.SR.C = false;
        // nes.SR.U = 1 nes.SR.B = true;
        nes.ram[0] = 0x08;
        emulate_instructions(nes, 1);
        assert_eq!(0xba, pop8(nes));
    }

    #[test]
    fn all_set() {
        let mut nes = &mut new_nes();
        nes.set_status_register(0xff);
        nes.ram[0] = 0x08;
        emulate_instructions(nes, 1);
        assert_eq!(0xff, pop8(nes));
    }
}
