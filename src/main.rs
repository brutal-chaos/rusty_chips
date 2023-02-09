/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use clap::Parser;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::watch;

pub(crate) mod chip8;
pub(crate) mod gui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CLI Arguments
    let args = Args::parse();
    let mut rom_file: File = File::open(args.rom).await?;
    let mut rom_bytes: Vec<u8> = Vec::new();
    rom_file.read_to_end(&mut rom_bytes).await?;

    // Comms Channels and async task prep
    let (send_alive, recv_alive) = watch::channel(true);
    let (send_input, recv_input) = watch::channel(0u8 as char);
    let (send_video, recv_video) = watch::channel([[false; 64]; 32]);
    let (send_vdclr, recv_vdclr) = watch::channel(false);
    let disp_task = tokio::spawn(gui::gui_loop(
        send_alive, send_input, recv_video, recv_vdclr,
    ));
    let cpu_task = tokio::spawn(chip8::chip8_runner(
        recv_alive,
        recv_input,
        send_video,
        send_vdclr,
        Some(rom_bytes),
    ));

    // Off to the races!
    let _ = tokio::join!(disp_task, cpu_task);
    Ok(())
}
