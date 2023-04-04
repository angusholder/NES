use criterion::{criterion_group, criterion_main, Criterion, BatchSize};

use nes_core::mapper::Mapper;
use nes_core::cartridge;
use nes_core::nes::NES;

use std::path::Path;


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "NROM Green Background",
        |b| b.iter_batched(
            || {
                let mut nes = load_nes_system("../samples/hello_green.nes");
                for _ in 0..60 { nes.simulate_frame(); }
                nes
            },
            |mut nes| {
                // Simulate a frame now that the NES system should be in steady-state
                nes.simulate_frame();
            },
            BatchSize::LargeInput,
        )
    );

    c.bench_function(
        "MMC1 Plumber Game",
        |b| b.iter_batched(
            || {
                let mut nes = load_nes_system("../ROMS/Super Mario Bros. (Japan, USA).nes");
                for _ in 0..60 { nes.simulate_frame(); }
                nes
            },
            |mut nes| {
                // Simulate a frame now that the NES system should be in steady-state
                nes.simulate_frame();
            },
            BatchSize::LargeInput,
        )
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn load_nes_system(
    filename: &str,
) -> Box<NES> {
    let cart = cartridge::parse_rom(Path::new(&filename)).unwrap();
    let mut nes = Box::new(NES::from_cart(cart).unwrap());
    nes.power_on();
    nes
}
