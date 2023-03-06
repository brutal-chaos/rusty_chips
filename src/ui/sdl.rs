/// ui/sdl.rs: interface between the OS and the emulator
/// Copyright (C) 2015-2023 Justin Noah <justinnoah+rusty_chips@gmail.com>

/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU Affero General Public License as published
/// by the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.

/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU Affero General Public License for more details.

/// You should have received a copy of the GNU Affero General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::collections::HashMap;

use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use log::debug;
use sdl2::{
    audio::AudioStatus,
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    video::{GLProfile, Window},
};

use crate::audio::init_sdl_audio;
use crate::chip8::Chip8Handle;
use crate::counter::CounterHandle;
use crate::fuse::FuseHandle;
use crate::input::InputHandle;
use crate::ui::{menus, types::PixelPanel};
use crate::vram::{ScreenSize, VRAMHandle};

fn glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|this| {
            window.subsystem().gl_get_proc_address(this) as _
        })
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

    // TODO: Make configurable
    let xf: f32 = 1280.0;
    let yf: f32 = 720.0;
    let xu: usize = 1280;
    let yu: usize = 720;
    let screen_size_pxu = (xu, yu);
    let _screen_size_pxf = (xf, yf);

    // TODO: Add SuperChip8 support too!
    let panel = match screen_size {
        ScreenSize::L => PixelPanel::new_large(screen_size_pxu.0, screen_size_pxu.1),
        ScreenSize::S => PixelPanel::new_small(screen_size_pxu.0, screen_size_pxu.1),
    };

    let sdl_context = sdl2::init().unwrap();
    let (_, audio_playback) = init_sdl_audio(&sdl_context);
    let video_sub = sdl_context.video().unwrap();
    let gl_attr = video_sub.gl_attr();
    gl_attr.set_context_profile(GLProfile::GLES);
    gl_attr.set_context_version(3, 2);

    let window = video_sub
        .window(
            "Rusty Chips",
            screen_size_pxu.0 as u32,
            screen_size_pxu.1 as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .target_texture()
        .build()
        .unwrap();

    let gl = glow_context(canvas.window());

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    let mut platform = SdlPlatform::init(&mut imgui);
    let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let menu_state = menus::MenuState::default();
    'running: loop {
        // Handle input
        for event in event_pump.poll_iter() {
            platform.handle_event(&mut imgui, &event);

            match event {
                Event::Quit { .. } => {
                    fuse.blow();
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    // draw menu
                    rt.block_on(async { c8.toggle_exec().await });
                    let mut show_menu_bar_handle = menu_state.show_menu_bar.write().unwrap();
                    *show_menu_bar_handle = !*show_menu_bar_handle;
                }
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
                _ => (),
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
        unsafe {
            let _ = sdl2::sys::SDL_RenderFlush(canvas.raw());
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

        if *menu_state.show_menu_bar.read().unwrap() {
            // draw menu
            platform.prepare_frame(&mut imgui, canvas.window(), &event_pump);
            let ui = imgui.new_frame();
            menus::main_menu(ui, &menu_state, fuse.clone());
            let draw_data = imgui.render();

            // Failures are ok
            renderer.render(draw_data).unwrap_or(());

            let mut rom_view = menu_state.rom_fs_view.chosen_rom.write().unwrap();
            if rom_view.len() > 0 {
                let local_copy_rom = rom_view.clone();
                rom_view.clear();
                let mut sub_menu_writer = menu_state.sub_window_opened.write().unwrap();
                *sub_menu_writer = false;
                let mut sub_window_writer = menu_state.show_menu_bar.write().unwrap();
                *sub_window_writer = false;

                rt.block_on(async {
                    video.clear_screen().await;
                    c8.load_rom(local_copy_rom).await;
                    c8.unpause().await;
                });
            } else {
                drop(rom_view);
            }
        } else {
            // We need the menu state to know we have notified the Chip8 to start executing again
            // First grab a write handle, we may need to change its value
            let mut running_with_scissors = *menu_state.pause_sent.write().unwrap();
            if !running_with_scissors {
                rt.block_on(async {
                    c8.unpause().await;
                });
                running_with_scissors = true;
            }
        }

        canvas.window().gl_swap_window();

        // Check fuse
        if !fuse.alive() {
            break 'running;
        }

        std::thread::sleep(std::time::Duration::from_secs_f64(0.00001));
    }
    debug!("Exiting GUI Task");
}
