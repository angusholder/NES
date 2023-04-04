use std::error::Error;
use std::io::Read;
use std::path::Path;
use log::{info};
use crate::mapper::MapperDescriptor;

pub fn parse_rom(filename: &Path) -> Result<Cartridge, Box<dyn Error>> {
    info!("Reading file: {}", filename.display());
    let mut file = std::fs::File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let Ok(header): Result<[u8; 16], _> = buffer[0..16].try_into() else {
        return Err("This doesn't appear to be a NES ROM".into());
    };
    if &header[..4] != b"NES\x1A" {
        return Err("This doesn't appear to be a NES ROM".into());
    }
    let mut rest = &buffer[16..];
    let mut prg_rom_size = header[4] as usize;
    info!("Header: {:?}", header);
    let mut chr_rom_size = header[5] as usize;
    let mut mapper_num: u32 = (header[6] as u32 >> 4) | (header[7] as u32 & 0xF0);

    let ines2 = header[7] & 0x0C == 0x08;

    let mut submapper_num: Option<u32> = None;
    let mut prg_ram_size: u32;
    let mut chr_ram_size: u32 = 0;

    // Extended version of the .nes file format - https://www.nesdev.org/wiki/NES_2.0
    if ines2 {
        mapper_num |= header[8] as u32 & 0x0F << 8;
        submapper_num = Some(header[8] as u32 >> 4);

        prg_rom_size |= (header[9] as usize & 0x0F) << 8;
        chr_rom_size |= (header[9] as usize & 0xF0) << 4;

        prg_ram_size = header[10] as u32 & 0x0F;
        if prg_ram_size != 0 {
            prg_ram_size = 64 << prg_ram_size;
        }
        let mut prg_nvram_size = header[10] as u32 >> 4;
        if prg_nvram_size != 0 {
            prg_nvram_size = 64 << prg_nvram_size;
        }
        chr_ram_size = header[11] as u32 & 0x0F;
        if chr_ram_size != 0 {
            chr_ram_size = 64 << chr_ram_size;
        }
        let mut chr_nvram_size = header[11] as u32 >> 4;
        if chr_nvram_size != 0 {
            chr_nvram_size = 64 << chr_nvram_size;
        }

        if chr_nvram_size != 0 || prg_nvram_size != 0 {
            return Err(format!("NV-RAM fields not supported (got CHR {chr_nvram_size}, PRG {prg_nvram_size})").into());
        }
    } else {
        if header[8] == 0 {
            // Value 0 implies 8 KB for compatibility
            // https://www.nesdev.org/wiki/INES#Flags_8
            prg_ram_size = 8 * 1024;
        } else {
            return Err(format!("Header 8 value {} not supported", header[8]).into());
        }
    }
    if chr_rom_size == 0 {
        chr_ram_size = 8192;
    }

    prg_rom_size *= 16 * 1024;
    chr_rom_size *= 8 * 1024;
    if rest.len() < prg_rom_size + chr_rom_size {
        return Err("This NES ROM appears to be invalid (too short)".into());
    }
    let prg_rom = &rest[..prg_rom_size];
    rest = &rest[prg_rom.len()..];
    let chr_rom = &rest[..chr_rom_size];
    rest = &rest[chr_rom.len()..];

    let chr: CHR = if chr_rom.is_empty() {
        assert_ne!(chr_ram_size, 0);
        CHR::RAM(chr_ram_size as usize)
    } else {
        assert_eq!(chr_ram_size, 0);
        CHR::ROM(chr_rom.to_vec().into_boxed_slice())
    };

    let Some(mapper_descriptor) = MapperDescriptor::for_number(mapper_num) else {
        return Err(format!("Mapper #{} not supported yet", mapper_num).into());
    };

    if let Some(submapper_num) = submapper_num {
        return Err(format!("Submapper field not supported. (got {submapper_num})").into());
    } else {
        info!("Mapper #{mapper_num}: {}", mapper_descriptor.name);
    }
    info!("PRG ROM size: {}K", prg_rom.len() / 1024);
    info!("PRG RAM size: {}K", prg_ram_size / 1024);
    match &chr {
        CHR::ROM(rom) => info!("CHR ROM {}K", rom.len() / 1024),
        CHR::RAM(ram_size) => info!("CHR RAM {}K", ram_size / 1024),
    }
    if !rest.is_empty() {
        info!("The file had {} tail bytes", rest.len());
    }

    let prg_ram_battery_backed = header[6] & 0b10 != 0;

    let mirroring = if header[6] & 0b1000 != 0 {
        NametableMirroring::FourScreen
    } else if header[6] & 0x01 == 0 {
        NametableMirroring::Horizontal
    } else {
        NametableMirroring::Vertical
    };

    Ok(Cartridge {
        mapper_descriptor,
        prg_ram_size,
        prg_ram_battery_backed,
        prg_rom: prg_rom.to_vec(),
        chr,
        mirroring,
    })
}

#[derive(Clone)]
pub struct Cartridge {
    pub mapper_descriptor: MapperDescriptor,
    pub prg_rom: Vec<u8>,
    pub chr: CHR,
    pub prg_ram_size: u32,
    pub prg_ram_battery_backed: bool,
    pub mirroring: NametableMirroring,
}

#[derive(Clone)]
pub enum CHR {
    RAM(usize),
    ROM(Box<[u8]>),
}

#[derive(Clone, Copy, Debug)]
pub enum NametableMirroring {
    Horizontal,
    Vertical,
    SingleScreenLowerBank,
    SingleScreenUpperBank,
    FourScreen,
}

#[test]
fn test_parse_rom() {
    parse_rom(Path::new("../samples/hello_green.nes")).unwrap();
}
