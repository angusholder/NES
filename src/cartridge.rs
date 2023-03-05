use std::error::Error;
use std::io::Read;
use std::path::Path;
use log::{info};

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
    let mut prg_ram_size: u32 = 0;
    let mut prg_nvram_size: u32 = 0;
    let mut chr_ram_size: u32 = 0;
    let mut chr_nvram_size: u32 = 0;

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
        prg_nvram_size = header[10] as u32 >> 4;
        if prg_nvram_size != 0 {
            prg_nvram_size = 64 << prg_nvram_size;
        }
        chr_ram_size = header[11] as u32 & 0x0F;
        if chr_ram_size != 0 {
            chr_ram_size = 64 << chr_ram_size;
        }
        chr_nvram_size = header[11] as u32 >> 4;
        if chr_nvram_size != 0 {
            chr_nvram_size = 64 << chr_nvram_size;
        }
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

    if let Some(submapper_num) = submapper_num {
        info!("Mapper #{mapper_num} (subtype {submapper_num})");
    } else {
        info!("Mapper #{mapper_num}");
    }
    info!("PRG ROM size: {}K", prg_rom.len() / 1024);
    info!("CHR ROM size: {}K", chr_rom.len() / 1024);
    info!("PRG RAM size: {prg_ram_size}");
    info!("PRG NVRAM size: {prg_nvram_size}");
    info!("CHR RAM size: {chr_ram_size}");
    info!("CHR NVRAM size: {chr_nvram_size}");
    if !rest.is_empty() {
        info!("The file had {} tail bytes", rest.len());
    }

    if header[6] & 0b1000 != 0 {
        return Err("Four-screen mirroring mode not supported".into());
    }
    let mirroring = if header[6] & 0x01 == 0 {
        NametableMirroring::Horizontal
    } else {
        NametableMirroring::Vertical
    };

    Ok(Cartridge {
        mapper_num,
        submapper_num,
        prg_ram_size,
        prg_nvram_size,
        chr_ram_size,
        chr_nvram_size,
        prg_rom: prg_rom.to_vec(),
        chr_rom: chr_rom.to_vec(),
        mirroring,
    })
}

pub struct Cartridge {
    pub mapper_num: u32,
    pub submapper_num: Option<u32>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_ram_size: u32,
    pub prg_nvram_size: u32,
    pub chr_ram_size: u32,
    pub chr_nvram_size: u32,
    pub mirroring: NametableMirroring,
}

#[derive(Clone, Copy, Debug)]
pub enum NametableMirroring {
    Horizontal,
    Vertical,
}

#[test]
fn test_parse_rom() {
    parse_rom(Path::new("samples/hello_green.nes")).unwrap();
}
