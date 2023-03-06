/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::string::String;
use std::sync::{Arc, RwLock};

use imgui::*;
use log::debug;

use crate::fuse::FuseHandle;

#[derive(Debug, Clone)]
pub struct FSListBox {
    idx: Arc<RwLock<i32>>,
    contents: Arc<RwLock<Vec<String>>>,
    cur_path: Arc<RwLock<String>>,
    cur_selected: Arc<RwLock<String>>,
    pub chosen_rom: Arc<RwLock<Vec<u8>>>,
}

impl FSListBox {
    fn new() -> Self {
        let new = Self {
            idx: Arc::new(RwLock::new(0)),
            contents: Arc::new(RwLock::new(vec![String::from("..")])),
            cur_path: Arc::new(RwLock::new(String::from(
                std::env::current_dir().unwrap().to_str().unwrap(), // home::home_dir().unwrap().as_path().to_str().unwrap(),
            ))),
            cur_selected: Arc::new(RwLock::new(String::from(""))),
            chosen_rom: Arc::new(RwLock::new(Vec::new())),
        };
        new.update_lists();
        new
    }

    fn update_lists(&self) {
        let cur_path_read_handle = self.cur_path.as_ref().read().unwrap();
        let path = PathBuf::from(String::from(&*cur_path_read_handle));
        let _ = path.join(&*self.cur_selected.as_ref().read().unwrap());
        let mut d = self.contents.as_ref().write().unwrap();
        d.clear();
        d.push(String::from(".."));

        for entry in read_dir(path).unwrap().flatten() {
            let name = String::from(entry.file_name().to_str().unwrap());
            d.push(name);
        }
        d.sort();
    }
}

// TODO: Remove dead code allowance
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MenuWindow {
    Game,
    Config,
    None,
}

#[derive(Debug, Clone)]
pub struct MenuState {
    // Whether to show a Config/LoadROM window
    pub open_window_type: Arc<RwLock<MenuWindow>>,
    // Data storage for the directory listing/rom selection window
    pub rom_fs_view: Arc<FSListBox>,
    // Invoke imgui at all?
    pub show_menu_bar: Arc<RwLock<bool>>,
    // A boolean modified by imgui to tell us the window is closed
    pub sub_window_opened: Arc<RwLock<bool>>,
    // we need to send a pause command to
    pub pause_sent: Arc<RwLock<bool>>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            // Init: No open window
            open_window_type: Arc::new(RwLock::new(MenuWindow::None)),
            // Init: see FSListBox for defaults
            rom_fs_view: Arc::new(FSListBox::new()),
            // Init: start with menubar closed
            show_menu_bar: Arc::new(RwLock::new(false)),
            // Init: neither Config/LoadROM are open at the start either
            sub_window_opened: Arc::new(RwLock::new(false)),
            // Don't send 'unpause' every frame
            pause_sent: Arc::new(RwLock::new(false)),
        }
    }
}

/// PLAYYING WITH FIRE (FFI BOUNDRIES)
pub fn main_menu(ui: &Ui, state: &MenuState, fuse: FuseHandle) {
    // see if imgui closed the sub window and set the current
    // sub window type to None if so
    let swo_h = *state.sub_window_opened.read().unwrap();
    let mut owt = *state.open_window_type.write().unwrap();
    if !swo_h && owt != MenuWindow::None {
        owt = MenuWindow::None;
    }
    drop(owt);
    drop(swo_h);

    let smb_h = *state.show_menu_bar.read().unwrap();
    let mut running = *state.pause_sent.write().unwrap();
    if !smb_h && running != true {
        running = true;
    }
    drop(running);
    drop(smb_h);

    ui.main_menu_bar(|| {
        ui.set_window_font_scale(2.0);
        ui.menu("ROM", || {
            if ui.menu_item("Load ROM") {
                let ow_arc = Arc::clone(&state.open_window_type);
                let mut ow = ow_arc.write().unwrap();
                *ow = MenuWindow::Game;
                let swo_arc = Arc::clone(&state.sub_window_opened);
                let mut swo = swo_arc.write().unwrap();
                *swo = true;
            }
            if ui.menu_item("Exit") {
                fuse.blow();
            }
            ui.set_window_font_scale(1.0);
        });

        match &*state.open_window_type.read().unwrap() {
            MenuWindow::Config => (),
            MenuWindow::Game => load_rom_window(ui, state),
            MenuWindow::None => config_window(ui, state),
        }
    });
}

/// PLAYYING WITH FIRE (FFI BOUNDRIES)
fn load_rom_window(ui: &Ui, state: &MenuState) {
    // Crossing one FFI boundry after another.
    // Playing it safe with memory.
    let _w = ui
        .window("Load ROM")
        .opened(&mut state.sub_window_opened.write().unwrap())
        .position([50.0, 50.0], Condition::FirstUseEver)
        .size([300.0, 600.0], Condition::FirstUseEver)
        .build(|| {
            let arc_path = Arc::clone(&state.rom_fs_view.cur_path);
            let arc_contents = Arc::clone(&state.rom_fs_view.contents);
            let arc_idx = Arc::clone(&state.rom_fs_view.idx);
            let arc_cur_sel = Arc::clone(&state.rom_fs_view.cur_selected);

            let _contents = arc_contents.read().unwrap().clone();
            let contents = _contents
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&'_ str>>();

            ui.text(&*arc_path.read().unwrap());
            if ui.list_box(
                "##directorylisting",
                &mut arc_idx.write().unwrap(),
                &contents,
                25,
            ) {
                let idx = arc_idx.read().unwrap();
                let mut current_selection = arc_cur_sel.write().unwrap();
                let p = PathBuf::from(arc_path.read().unwrap().clone())
                    .join(contents[*idx as usize])
                    .canonicalize()
                    .unwrap();

                current_selection.clear();
                if p.is_dir() {
                    let mut path = arc_path.write().unwrap();
                    path.clear();
                    for c in String::from(p.as_path().to_str().unwrap()).chars() {
                        path.push(c);
                    }
                    drop(path);
                    drop(current_selection);
                    drop(idx);
                    state.rom_fs_view.update_lists()
                } else {
                    let sttr = p.file_name().unwrap().to_str().unwrap();
                    for c in String::from(sttr).chars() {
                        current_selection.push(c);
                    }
                    drop(current_selection);
                    drop(idx);
                }
            }

            // "if let Some(_) ..." form is needed by .build(..)
            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_table("##ListAndLoadFileTable", 2) {
                let arc_chosen_rom = Arc::clone(&state.rom_fs_view.chosen_rom);
                ui.table_next_row();
                ui.table_set_column_index(0);
                let cursel = arc_cur_sel.read().unwrap();
                ui.text(&*cursel);
                drop(cursel);
                let mut cursel = arc_cur_sel.write().unwrap();
                if !cursel.is_empty() {
                    let mut chosen_rom = arc_chosen_rom.as_ref().write().unwrap();
                    ui.table_set_column_index(1);
                    if ui.button("Load") {
                        chosen_rom.clear();
                        let p = PathBuf::from(&*arc_path.read().unwrap()).join(&*cursel);
                        match File::open(p) {
                            Ok(mut file) => {
                                let _ = file.read_to_end(&mut chosen_rom).unwrap();
                            }
                            _ => *cursel = String::from("Unable to load file"),
                        }
                    }
                    drop(chosen_rom);
                }
                drop(cursel);
            }
        });
}

/// (WILL BE) PLAYYING WITH FIRE (FFI BOUNDRIES)
fn config_window(_ui: &Ui, _state: &MenuState) {}
