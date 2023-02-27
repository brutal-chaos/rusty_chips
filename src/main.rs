/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::fs::File;
use std::io::Read;

use clap::Parser;

use fuse::FuseHandle;
use input::InputHandle;
use vram::{ScreenSize, VRAMHandle};

pub(crate) mod chip8;
pub(crate) mod counter;
pub(crate) mod fuse;
pub(crate) mod gui;
pub(crate) mod input;
pub(crate) mod util;
pub(crate) mod vram;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom: Option<String>,
    #[arg(short, long, default_value = "1.76Mhz")]
    speed: Option<String>,
}

fn cli_args() -> (Vec<u8>, f64) {
    // CLI Arguments
    let args = Args::parse();
    let rom: Vec<u8> = match args.rom.as_deref() {
        Some(path) => {
            let mut r = File::open(path).unwrap();
            let mut v = Vec::new();
            r.read_to_end(&mut v).unwrap();
            v
        }
        None => {
            let roms = util::test_roms();
            let rom = roms[0].clone();
            rom
        }
    };
    let mut cpu_speed: f64 = 0.0;
    if let Some(speed) = args.speed.as_deref() {
        cpu_speed = util::hz_to_secs(speed);
    } else {
        // Original COSMAC VIP Frequency
        cpu_speed = util::hz_to_secs("1.76MHz");
    }
    (rom, cpu_speed)
}

fn main() {
    let (rom, freq) = cli_args();

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Comms Channels and async task prep
    let (video, input, fuse) = rt.block_on(async {
        (
            VRAMHandle::new(ScreenSize::S),
            InputHandle::new(),
            FuseHandle::new(),
        )
    });
    let _chip8_handle = rt.block_on(async {
        chip8::Chip8Handle::new(freq, Some(rom), input.clone(), video.clone(), fuse.clone()).await
    });

    gui::gui_loop(
        fuse.clone(),
        input.clone(),
        video.clone(),
        ScreenSize::S,
        rt.handle(),
    );
}
