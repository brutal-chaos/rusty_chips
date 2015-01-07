extern crate sdl;

use std::rand::Rng;

use sdl::event::{Event, Key};
use sdl::video::{SurfaceFlag, VideoFlag};

struct Chip8Unit {
    /*
     * 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
     * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
     * 0x200-0xFFF - Program ROM and work RAM
     */
    memory:    [u8; 4096],

    /*
     * CPU registers: The Chip 8 has 15 8-bit general purpose registers
     * named V0,V1 up to VE. The 16th register is used  for the ‘carry flag’.
     */
    registers: [u8; 16],

    // Index register
    i: u16,

    // program counter
    pc: u16,

    // Screen - 64 x 32 pixels
    graphics: [u8; 64*32],

    // No timers, but 60hz counters
    delay_timer: u8,
    sound_timer: u8,

    // Stack needed to jump to certain addresses or call subroutines
    // The chip8 stack has 16 levels
    stack_pointer: u16,
    stack: [u16; 16],

    // Chip8 keypad had 16 keys - keep track of the kb state
    keypad: [u8; 16],
}

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
