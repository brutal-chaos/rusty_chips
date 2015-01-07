// externs
extern crate sdl;

// uses
use std::rand::Rng;

use sdl::event::{Event, Key};
use sdl::video::{SurfaceFlag, VideoFlag};

use chip8::Chip8;

// mods
mod chip8;

// Finally, some code

fn init_sdl() {
    // Initialize SDL Video
    sdl::init([sdl::InitFlag::Video].as_slice());

    // Give our window a title
    sdl::wm::set_caption("rusty_chips a Chip8 emulator", "rusty_chips");

    // Initialize the screen
    let screen = match sdl::video::set_video_mode(800, 600, 32,
                                                  [SurfaceFlag::HWSurface].as_slice(),
                                                  [VideoFlag::DoubleBuf].as_slice()) {
        Ok(screen) => screen,
        Err(err) => panic!("Failed to set video mode: {}", err)
    };
}

fn main() {
    // Initialize SDL
    init_sdl();

    // SDL main loop
    'main : loop {
        'event : loop {
            match sdl::event::poll_event() {
                Event::Quit => break 'main,
                Event::None => break 'event,
                Event::Key (k,_,_,_)
                    if k == Key::Escape
                        => break 'main,
                    _ => {}
            }
        }
    }

    // Quit
    sdl::quit();
}
