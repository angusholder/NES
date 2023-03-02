use std::error::Error;
use std::io::Read;

pub fn parse_rom(filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let header = &buffer[0..16];
    let mut rest = &buffer[16..];
    let prg_rom_size = header[4] as usize;
    println!("Header: {:?}", header);
    println!("PRG ROM size: {} x 16K", prg_rom_size);
    let chr_rom_size = header[5] as usize;
    println!("CHR ROM size: {} x 8K", chr_rom_size);
    let flags_6 = header[6];
    if flags_6 != 0 {
        panic!("flags_6 not supported: {:02X}", flags_6);
    }
    let flags_7 = header[7];
    if flags_7 != 0 {
        panic!("flags_7 not supported: {:02X}", flags_7);
    }
    if header[8] != 0 {
        panic!("Mapper not supported: {:02X}", header[8]);
    }

    let prg_rom = &rest[..prg_rom_size * 16 * 1024];
    rest = &rest[prg_rom.len()..];
    let chr_rom = &rest[..chr_rom_size * 8 * 1024];
    rest = &rest[chr_rom.len()..];

    println!("PRG ROM size: {}", prg_rom.len());
    println!("CHR ROM size: {}", chr_rom.len());
    println!("Rest size: {}", rest.len());

    Ok(())
}

#[test]
fn test_parse_rom() {
    parse_rom("samples/hello_green.nes").unwrap();
}
