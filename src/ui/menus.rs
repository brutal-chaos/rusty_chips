/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::fs::read_dir;
use std::path::PathBuf;
use std::string::String;
use std::sync::{Arc, RwLock};

use imgui::*;

use crate::fuse::FuseHandle;

#[derive(Debug, Clone)]
pub struct FSListBox {
    idx: Arc<RwLock<i32>>,
    contents: Arc<RwLock<Vec<String>>>,
    cur_path: Arc<RwLock<String>>,
    cur_selected: Arc<RwLock<String>>,
}

impl FSListBox {
    fn new() -> Self {
        let new = Self {
            idx: Arc::new(RwLock::new(0)),
            contents: Arc::new(RwLock::new(vec![String::from("..")])),
            cur_path: Arc::new(RwLock::new(String::from(
                home::home_dir().unwrap().as_path().to_str().unwrap(),
            ))),
            cur_selected: Arc::new(RwLock::new(String::from(""))),
        };
        new.update_lists();
        new
    }

    fn update_lists(&self) {
        let cur_path_read_handle = self.cur_path.as_ref().read().unwrap();
        let string_path = String::from(&*cur_path_read_handle);
        let path = PathBuf::try_from(string_path).unwrap();
        let cur_selected_read_handle = self.cur_selected.as_ref().read().unwrap();
        let _ = path.join(&*cur_selected_read_handle);
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
#[derive(Debug, Copy, Clone)]
pub enum MenuWindow {
    Game,
    Config,
    None,
}

#[derive(Debug, Clone)]
pub struct MenuState {
    pub open_window: Arc<RwLock<MenuWindow>>,
    pub sub_window_opened: Arc<RwLock<bool>>,
    pub show_menu: Arc<RwLock<bool>>,
    pub game_fs_view: Arc<FSListBox>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            open_window: Arc::new(RwLock::new(MenuWindow::None)),
            sub_window_opened: Arc::new(RwLock::new(false)),
            show_menu: Arc::new(RwLock::new(false)),
            game_fs_view: Arc::new(FSListBox::new()),
        }
    }
}

pub fn main_menu(ui: &Ui, state: &MenuState, fuse: FuseHandle) {
    if !*state.sub_window_opened.read().unwrap() {
        let mut l = state.open_window.write().unwrap();
        *l = MenuWindow::None;
    }

    ui.main_menu_bar(|| {
        ui.set_window_font_scale(2.0);
        ui.menu("ROM", || {
            if ui.menu_item("Load ROM") {
                let ow_arc = Arc::clone(&state.open_window);
                let mut ow = ow_arc.write().unwrap();
                *ow = MenuWindow::Game;
                let swo_arc = Arc::clone(&state.sub_window_opened);
                let mut swo = swo_arc.write().unwrap();
                *swo = true;
            }
            ui.menu_item("Close ROM");
            if ui.menu_item("Exit") {
                fuse.blow();
            }
            ui.set_window_font_scale(1.0);
        });

        match &*state.open_window.read().unwrap() {
            MenuWindow::Config => (),
            MenuWindow::Game => load_game_window(ui, state),
            MenuWindow::None => config_window(ui, state),
        }
    });
}

fn load_game_window(ui: &Ui, state: &MenuState) {
    let _w = ui
        .window("Load ROM")
        .opened(&mut state.sub_window_opened.write().unwrap())
        .position([50.0, 50.0], Condition::FirstUseEver)
        .size([300.0, 600.0], Condition::FirstUseEver)
        .build(|| {
            let arc_path = Arc::clone(&state.game_fs_view.cur_path);
            let arc_contents = Arc::clone(&state.game_fs_view.contents);
            let arc_idx = Arc::clone(&state.game_fs_view.idx);
            let arc_cur_sel = Arc::clone(&state.game_fs_view.cur_selected);

            let _contents = arc_contents.read().unwrap().clone();
            let contents = _contents.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

            ui.text(&*arc_path.read().unwrap());
            if ui.list_box(
                "##directorylisting",
                &mut arc_idx.write().unwrap(),
                &contents,
                25,
            ) {
                let idx = arc_idx.read().unwrap();
                let mut w = arc_cur_sel.write().unwrap();
                let p = PathBuf::from(arc_path.read().unwrap().clone())
                    .join(contents[*idx as usize])
                    .canonicalize()
                    .unwrap();

                w.clear();
                if p.is_dir() {
                    let mut path = arc_path.write().unwrap();
                    path.clear();
                    for c in String::from(p.as_path().to_str().unwrap()).chars() {
                        path.push(c);
                    }
                    drop(path);
                    drop(w);
                    drop(idx);
                    state.game_fs_view.update_lists()
                } else {
                    let sttr = p.file_name().unwrap().to_str().unwrap();
                    for c in String::from(sttr).chars() {
                        w.push(c);
                    }
                    drop(w);
                    drop(idx);
                }
            }

            // "if let Some(_) ..." form is needed by .build(..)
            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = ui.begin_table("##ListAndLoadFileTable", 2) {
                ui.table_next_row();
                ui.table_set_column_index(0);
                let cursel = arc_cur_sel.read().unwrap();
                ui.text(&*cursel);
                if !cursel.is_empty() {
                    ui.table_set_column_index(1);
                    if ui.button("Load") {}
                }
            }
        });
}

fn config_window(_ui: &Ui, _state: &MenuState) {}
