/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::sync::{Arc, RwLock};
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

pub(crate) mod chip8;

async fn gui_loop(alive_lock: Arc<RwLock<bool>>) {
    let sdl_context = sdl2::init().unwrap();
    let video_sub = sdl_context.video().unwrap();
    let window = video_sub
        .window("Rusty Chips", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    let mut alive = alive_lock.write().unwrap();
                    *alive = false;
                    break 'running;
                }
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let alive_lock = Arc::new(RwLock::new(true));
    let disp_alive = alive_lock.clone();
    let disp_task = tokio::spawn(gui_loop(disp_alive));
    let mut tasks = Vec::with_capacity(2);
    tasks.push(disp_task);
    for task in tasks {
        task.await.unwrap();
    }
    Ok(())
}
