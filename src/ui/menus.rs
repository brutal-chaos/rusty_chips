/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use imgui::{Condition, Ui};
use log::debug;

use crate::fuse::FuseHandle;

pub fn main_menu(ui: &Ui, fuse: FuseHandle) {
    let w = ui.main_menu_bar(|| {
        if ui.button("Game") {
            debug!("Open Game Submenu");
            open_game_window(ui);
        }
        ui.spacing();
        ui.spacing();
        ui.spacing();
        if ui.button("Config") {
            debug!("Open Config Submenu");
            open_config_window(ui);
        }
        ui.spacing();
        ui.spacing();
        ui.spacing();
        if ui.button("Quit") {
            fuse.blow();
        }
    });
}

fn open_game_window(_ui: &Ui) {}
fn open_config_window(_ui: &Ui) {}
