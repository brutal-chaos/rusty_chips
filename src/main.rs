/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
// externs
extern crate sdl;

// uses
use std::default::Default;

use sdl::event::{Event, Key};
use sdl::video::{SurfaceFlag, VideoFlag};

use chip8::Chip8;

// mods
mod chip8;

/* Finally, some code */

fn init_sdl() {
    // Initialize SDL Video
    sdl::init([sdl::InitFlag::Video].as_slice());

    // Give our window a title
    sdl::wm::set_caption("rusty_chips a Chip8 emulator", "rusty_chips");

    // Initialize the screen
    let screen = match sdl::video::set_video_mode(
        800,
        600,
        32,
        [SurfaceFlag::HWSurface].as_slice(),
        [VideoFlag::DoubleBuf].as_slice(),
    ) {
        Ok(screen) => screen,
        Err(err) => panic!("Failed to set video mode: {}", err),
    };
}

fn init_chip8() -> chip8::Chip8 {
    let mut vm = Chip8 {
        ..Default::default()
    };

    // Fontset
    let fontset = [
        0xF0u8, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    for c in range(0, fontset.len()) {
        vm.memory[c] = fontset[c];
    }

    vm
}

fn main() {
    // Initialize the Chip8
    let mut vm = init_chip8();

    // Initialize SDL
    init_sdl();

    // SDL main loop
    'main: loop {
        'event: loop {
            match sdl::event::poll_event() {
                Event::Quit => break 'main,
                Event::None => break 'event,
                Event::Key(k, _, _, _) if k == Key::Escape => break 'main,
                _ => {}
            }
        }
    }

    // Quit
    sdl::quit();
}
