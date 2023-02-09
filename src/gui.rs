/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

fn init_rects(sx: usize, sy: usize) -> [[Rect; 64]; 32] {
    let screen: [[Rect; 64]; 32] = {
        let mut s: Vec<[Rect; 64]> = Vec::with_capacity(32);
        let square_width = (sx / 64) as u32;
        let square_height = (sy / 32) as u32;
        for ty in 0..32 {
            let row: [Rect; 64] = {
                let mut m: Vec<Rect> = Vec::with_capacity(64);
                for tx in 0..64 {
                    let x = (tx * square_width) as i32;
                    let y = (ty * square_height) as i32;
                    let rect = Rect::new(x, y, square_width, square_height);
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

    screen
}

pub async fn gui_loop(
    alive: tokio::sync::watch::Sender<bool>,
    input: tokio::sync::watch::Sender<char>,
    video: tokio::sync::watch::Receiver<[[bool; 64]; 32]>,
    vdclr: tokio::sync::watch::Receiver<bool>,
) {
    println!("Start GUI Task");
    let sdl_context = sdl2::init().unwrap();
    let video_sub = sdl_context.video().unwrap();
    let window = video_sub
        .window("Rusty Chips", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let screen = init_rects(800, 600);
    let mut onoffs = *video.borrow();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::BLUE);
    canvas.clear();
    canvas.present();

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    alive.send(false).unwrap_or(());
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Num1 => {
                        println!("Keypressed!");
                        input.send(1u8 as char).unwrap_or(());
                    }
                    Keycode::Num2 => {
                        input.send(2u8 as char).unwrap_or(());
                    }
                    Keycode::Num3 => {
                        input.send(3u8 as char).unwrap_or(());
                    }
                    Keycode::Num4 => {
                        input.send(0xC as char).unwrap_or(());
                    }
                    Keycode::Q => {
                        input.send(4u8 as char).unwrap_or(());
                    }
                    Keycode::W => {
                        input.send(5u8 as char).unwrap_or(());
                    }
                    Keycode::E => {
                        input.send(6u8 as char).unwrap_or(());
                    }
                    Keycode::R => {
                        input.send(0xD as char).unwrap_or(());
                    }
                    Keycode::A => {
                        input.send(7u8 as char).unwrap_or(());
                    }
                    Keycode::S => {
                        input.send(8u8 as char).unwrap_or(());
                    }
                    Keycode::D => {
                        input.send(9u8 as char).unwrap_or(());
                    }
                    Keycode::F => {
                        input.send(0xE as char).unwrap_or(());
                    }
                    Keycode::Z => {
                        input.send(0xA as char).unwrap_or(());
                    }
                    Keycode::X => {
                        input.send(0u8 as char).unwrap_or(());
                    }
                    Keycode::C => {
                        input.send(0xB as char).unwrap_or(());
                    }
                    Keycode::V => {
                        input.send(0xF as char).unwrap_or(());
                    }
                    _ => (),
                },
                Event::KeyUp { .. } => {
                    input.send(0u8 as char).unwrap_or(());
                }
                _ => {}
            }
        }
        onoffs = *video.borrow();
        for (y, row) in screen.iter().enumerate() {
            for (x, rect) in row.iter().enumerate() {
                if onoffs[y][x] {
                    canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                } else {
                    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                }
                canvas.fill_rect(*rect).unwrap();
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    println!("Exiting GUI Task");
}
