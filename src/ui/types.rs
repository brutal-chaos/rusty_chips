/// ui/types.rs: sdl representation of the chip8 video memory
/// Copyright (C) 2023 Justin Noah <justinnoah+rusty_chips@gmail.com>

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
use std::ops::{Index, IndexMut};

use sdl2::rect::Rect;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum Pixels {
    l(Box<[[Rect; 128]; 64]>),
    s(Box<[[Rect; 64]; 32]>),
}

#[allow(non_snake_case)]
impl Pixels {
    pub fn L(width: usize, height: usize) -> Self {
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
        Pixels::l(Box::new(mem))
    }

    pub fn S(pixel_width: usize, pixel_height: usize) -> Self {
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
        Pixels::s(Box::new(mem))
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
    pub width: usize,
    pub height: usize,
    pub mem: Pixels,
}

impl PixelPanel {
    pub fn new_large(screen_width: usize, screen_height: usize) -> Self {
        PixelPanel {
            width: 128,
            height: 64,
            mem: Pixels::L(screen_width, screen_height),
        }
    }

    pub fn new_small(screen_width: usize, screen_height: usize) -> Self {
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
