/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::fs::File;
use std::io::Read;

use clap::Parser;

use chip8::Chip8Handle;
use fuse::FuseHandle;
use input::InputHandle;
use vram::{ScreenSize, VRAMHandle};

pub(crate) mod audio;
pub(crate) mod chip8;
pub(crate) mod counter;
pub(crate) mod fuse;
pub(crate) mod input;
pub(crate) mod ui;
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
            roms[0].clone()
        }
    };

    let cpu_speed: f64 = {
        if let Some(speed) = args.speed.as_deref() {
            util::hz_to_secs(speed)
        } else {
            // Original COSMAC VIP Frequency
            util::hz_to_secs("1.76MHz")
        }
    };

    (rom, cpu_speed)
}

fn main() {
    simple_logger::init_with_env().unwrap();

    let (rom, freq) = cli_args();

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Comms Channels and async task prep
    let (video, input, fuse, chip8, audio) = rt.block_on(async {
        let video = VRAMHandle::new(ScreenSize::S);
        let input = InputHandle::new();
        let fuse = FuseHandle::new();
        let chip8 = Chip8Handle::new(freq, Some(rom), input.clone(), video.clone(), fuse.clone());
        let audio_timer = chip8.sound_timer.clone();
        (video, input, fuse, chip8, audio_timer)
    });

    ui::gui_loop(fuse, input, video, audio, chip8, ScreenSize::S, rt.handle());
}
