/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::time::Duration;

use log::debug;
use sdl2::audio::AudioStatus;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::audio::init_sdl_audio;
use crate::chip8::Chip8Handle;
use crate::counter::CounterHandle;
use crate::fuse::FuseHandle;
use crate::input::InputHandle;
use crate::vram::{ScreenSize, VRAMHandle};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
enum Pixels {
    l([[Rect; 128]; 64]),
    s([[Rect; 64]; 32]),
}

#[allow(non_snake_case)]
impl Pixels {
    fn L(width: usize, height: usize) -> Self {
        let mem: [[Rect; 128]; 64] = {
            let mut s: Vec<[Rect; 128]> = Vec::with_capacity(64);
            let square_width = (width / 128) as u32;
            let square_height = (height / 64) as u32;
            for row_id in 0..64 {
                let row: [Rect; 128] = {
                    let mut m: Vec<Rect> = Vec::with_capacity(128);
                    for col_id in 0..128 {
                        let x = (col_id * square_width) as i32;
                        let y = (row_id * square_height) as i32;
                        let rect = Rect::new(x, y, square_width, square_height);
                        m.push(rect);
                    }
                    m.try_into().unwrap_or_else(|v: Vec<Rect>| {
                        panic!("expected vec of len {}, but found {}", 128, v.len())
                    })
                };
                s.push(row);
            }
            s.try_into().unwrap_or_else(|v: Vec<[Rect; 128]>| {
                panic!("expected vec of len {}, but found {}", 64, v.len())
            })
        };
        Pixels::l(mem)
    }

    fn S(pixel_width: usize, pixel_height: usize) -> Self {
        let mem: [[Rect; 64]; 32] = {
            let mut s: Vec<[Rect; 64]> = Vec::with_capacity(32);
            let square_pixel_width = (pixel_width / 64) as u32;
            let square_pixel_height = (pixel_height / 32) as u32;
            for row_id in 0..32 {
                let row: [Rect; 64] = {
                    let mut m: Vec<Rect> = Vec::with_capacity(64);
                    for col_id in 0..64 {
                        let x = (col_id * square_pixel_width) as i32;
                        let y = (row_id * square_pixel_height) as i32;
                        let rect = Rect::new(x, y, square_pixel_width, square_pixel_height);
                        m.push(rect);
                    }
                    m.try_into().unwrap_or_else(|v: Vec<Rect>| {
                        panic!("expected vec of len {}, but found {}", 64, v.len())
                    })
                };
                s.push(row);
            }
            s.try_into().unwrap_or_else(|v: Vec<[Rect; 64]>| {
                panic!("expected vec of len {}, but found {}", 32, v.len())
            })
        };
        Pixels::s(mem)
    }
}

impl Index<(usize, usize)> for Pixels {
    type Output = Rect;

    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        match self {
            Pixels::l(scrn) => &scrn[pos.1][pos.0],
            Pixels::s(scrn) => &scrn[pos.1][pos.0],
        }
    }
}

impl IndexMut<(usize, usize)> for Pixels {
    fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
        match self {
            Pixels::l(scrn) => &mut scrn[pos.1][pos.0],
            Pixels::s(scrn) => &mut scrn[pos.1][pos.0],
        }
    }
}

#[derive(Debug)]
pub struct PixelPanel {
    width: usize,
    height: usize,
    mem: Pixels,
}

impl PixelPanel {
    fn new_large(screen_width: usize, screen_height: usize) -> Self {
        PixelPanel {
            width: 128,
            height: 64,
            mem: Pixels::L(screen_width, screen_height),
        }
    }

    fn new_small(screen_width: usize, screen_height: usize) -> Self {
        PixelPanel {
            width: 64,
            height: 32,
            mem: Pixels::S(screen_width, screen_height),
        }
    }
}

impl Index<(usize, usize)> for PixelPanel {
    type Output = Rect;

    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        &self.mem[pos]
    }
}

impl IndexMut<(usize, usize)> for PixelPanel {
    fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
        &mut self.mem[pos]
    }
}

pub fn gui_loop(
    fuse: FuseHandle,
    input: InputHandle,
    video: VRAMHandle,
    sound_timer: CounterHandle,
    c8: Chip8Handle,
    screen_size: ScreenSize,
    rt: &tokio::runtime::Handle,
) {
    debug!("Start GUI");
    // TODO: Make configurable
    let screen_size_px = (1280usize, 720usize);
    // Hardcoded Keys, TODO: Make configurable
    let keycodes = HashMap::from([
        (Keycode::Num1, 0x1u8),
        (Keycode::Num2, 0x2u8),
        (Keycode::Num3, 0x3u8),
        (Keycode::Num4, 0xCu8),
        (Keycode::Q, 0x4u8),
        (Keycode::W, 0x5u8),
        (Keycode::E, 0x6u8),
        (Keycode::R, 0xDu8),
        (Keycode::A, 0x7u8),
        (Keycode::S, 0x8u8),
        (Keycode::D, 0x9u8),
        (Keycode::F, 0xEu8),
        (Keycode::Z, 0xAu8),
        (Keycode::X, 0x0u8),
        (Keycode::C, 0xBu8),
        (Keycode::V, 0xFu8),
    ]);

    let sdl_context = sdl2::init().unwrap();
    let video_sub = sdl_context.video().unwrap();
    let (_, audio_playback) = init_sdl_audio(&sdl_context);

    let window = video_sub
        .window(
            "Rusty Chips",
            screen_size_px.0 as u32,
            screen_size_px.1 as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let panel = match screen_size {
        ScreenSize::L => PixelPanel::new_large(screen_size_px.0, screen_size_px.1),
        ScreenSize::S => PixelPanel::new_small(screen_size_px.0, screen_size_px.1),
    };
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::BLUE);
    canvas.clear();
    canvas.present();

    'running: loop {
        canvas.clear();

        // Handle input
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    fuse.blow();
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => rt.block_on(async { c8.toggle_pause().await }),
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let key = *keycodes.get(&key).unwrap_or(&255u8);
                    if key != 255u8 {
                        rt.block_on(async { input.key_down(key).await });
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let key = *keycodes.get(&key).unwrap_or(&255u8);
                    if key != 255u8 {
                        rt.block_on(async { input.key_up(key).await });
                    }
                }
                _ => {}
            }
        }

        // Update Video
        let vram = rt.block_on(async { video.get().await });
        for x in 0..panel.width {
            for y in 0..panel.height {
                if vram[(x, y)] {
                    canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                } else {
                    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                }
                canvas.fill_rect(panel[(x, y)]).unwrap();
            }
        }

        // Update Audio
        rt.block_on(async {
            let status = audio_playback.status();
            let count: u8 = sound_timer.get().await;
            if count > 0 {
                match status {
                    AudioStatus::Paused | AudioStatus::Stopped => {
                        // Start playback
                        audio_playback.resume();
                    }
                    AudioStatus::Playing => (),
                }
            } else {
                match status {
                    AudioStatus::Paused | AudioStatus::Stopped => (),
                    AudioStatus::Playing => {
                        // Stop playback
                        audio_playback.pause();
                    }
                }
            }
        });

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    debug!("Exiting GUI Task");
}
