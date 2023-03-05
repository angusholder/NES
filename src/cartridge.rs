use std::error::Error;
use std::io::Read;
use std::path::Path;
use log::{info, warn};

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
    let prg_rom_size = header[4] as usize * 16 * 1024;
    info!("Header: {:?}", header);
    let chr_rom_size = header[5] as usize * 8 * 1024;
    let mapper_num = (header[6] >> 4) | (header[7] & 0xF0);
    info!("Mapper: {}", mapper_num);

    if rest.len() < prg_rom_size + chr_rom_size {
        return Err("This NES ROM appears to be invalid (too short)".into());
    }
    let prg_rom = &rest[..prg_rom_size];
    rest = &rest[prg_rom.len()..];
    let chr_rom = &rest[..chr_rom_size];
    rest = &rest[chr_rom.len()..];

    info!("PRG ROM size: {}", prg_rom.len());
    info!("CHR ROM size: {}", chr_rom.len());
    info!("Rest size: {}", rest.len());

    if header[6] & 0b1000 != 0 {
        warn!("Four-screen mirroring not supported");
    }
    let mirroring = if header[6] & 0x01 == 0 {
        NametableMirroring::Horizontal
    } else {
        NametableMirroring::Vertical
    };

    Ok(Cartridge {
        mapper_num,
        prg_rom: prg_rom.to_vec(),
        chr_rom: chr_rom.to_vec(),
        mirroring,
    })
}

pub struct Cartridge {
    pub mapper_num: u8,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
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
