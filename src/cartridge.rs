use std::error::Error;
use std::io::Read;
use std::path::Path;
use log::{info, warn};

pub fn parse_rom(filename: &Path) -> Result<Cartridge, Box<dyn Error>> {
    info!("Reading file: {}", filename.display());
    let mut file = std::fs::File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let header: [u8; 16] = buffer[0..16].try_into().unwrap();
    let mut rest = &buffer[16..];
    let prg_rom_size = header[4] as usize;
    info!("Header: {:?}", header);
    info!("PRG ROM size: {} x 16K", prg_rom_size);
    let chr_rom_size = header[5] as usize;
    info!("CHR ROM size: {} x 8K", chr_rom_size);
    let flags_6 = header[6];
    if flags_6 != 0 {
        warn!("flags_6 not supported: {:02X}", flags_6);
    }
    let flags_7 = header[7];
    if flags_7 != 0 {
        warn!("flags_7 not supported: {:02X}", flags_7);
    }
    if header[8] != 0 {
        warn!("Mapper not supported: {:02X}", header[8]);
    }
    let mapper_num = (header[6] >> 4) | (header[7] & 0xF0);
    info!("Mapper: {}", mapper_num);

    let prg_rom = &rest[..prg_rom_size * 16 * 1024];
    rest = &rest[prg_rom.len()..];
    let chr_rom = &rest[..chr_rom_size * 8 * 1024];
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