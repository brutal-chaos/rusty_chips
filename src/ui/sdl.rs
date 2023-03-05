/// Copyright 2015-2023, Justin Noah <justinnoah t gmail.com>, All Rights Reserved
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
                    rt.block_on(async { c8.toggle_pause().await });
                    let mut show_menu_handle = menu_state.show_menu.write().unwrap();
                    *show_menu_handle = !*show_menu_handle;
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

        unsafe {
            let _ = sdl2::sys::SDL_RenderFlush(canvas.raw());
        }
        if *menu_state.show_menu.read().unwrap() {
            // draw menu
            platform.prepare_frame(&mut imgui, canvas.window(), &event_pump);
            let ui = imgui.new_frame();
            menus::main_menu(ui, &menu_state, fuse.clone());
            let draw_data = imgui.render();

            // Failures are ok
            renderer.render(draw_data).unwrap_or(());
        }
        canvas.window().gl_swap_window();

        // Check fuse
        if !fuse.alive() {
            break 'running;
        }

        // std::thread::sleep(std::time::Duration::from_secs_f64(0.00001));
    }
    debug!("Exiting GUI Task");
}
