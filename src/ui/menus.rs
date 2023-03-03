/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::default::Default;

use imgui::*;

use crate::fuse::FuseHandle;

// TODO: Remove dead code allowance
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum MenuWindow {
    Game,
    Config,
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct MenuState {
    pub open_window: MenuWindow,
    pub sub_window_opened: bool,
    pub show_menu: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            open_window: MenuWindow::None,
            sub_window_opened: false,
            show_menu: false,
        }
    }
}

pub fn main_menu(ui: &Ui, state: &mut MenuState, fuse: FuseHandle) {
    if !state.sub_window_opened {
        state.open_window = MenuWindow::None;
    }

    ui.main_menu_bar(|| {
        ui.set_window_font_scale(2.0);
        ui.menu("Game", || {
            if ui.menu_item("Open Game") {
                state.open_window = MenuWindow::Game;
                state.sub_window_opened = true;
            }
            ui.menu_item("Close Game");
            if ui.menu_item("Exit") {
                fuse.blow();
            }
            ui.set_window_font_scale(1.0);
        });

        match &state.open_window {
            MenuWindow::Config => (),
            MenuWindow::Game => game_window(ui, state),
            MenuWindow::None => config_window(ui, state),
        }
    });
}

fn game_window(ui: &Ui, state: &mut MenuState) {
    let _w = ui
        .window("Open Game")
        .opened(&mut state.sub_window_opened)
        .position([50.0, 50.0], Condition::FirstUseEver)
        .size([600.0, 600.0], Condition::FirstUseEver)
        .build(|| {
            if let Some(_t) = ui.begin_table("Open", 3) {
                ui.table_next_row();

                ui.table_set_column_index(0);
                ui.text("Current Path");
                ui.table_next_column();
                let mut s = String::from("/");
                let _ = ui.input_text("Path", &mut s).build();
                ui.table_next_column();
                ui.button("Update");
                // Wrap around
                ui.table_next_column();

                // Next row 2nd column (1st element given 0-indexing)
                ui.table_next_column();
                ui.text("Files and stuff");
                ui.table_next_column();
                ui.new_line();
            }
        });
}

fn config_window(_ui: &Ui, _state: &mut MenuState) {}
