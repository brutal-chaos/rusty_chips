/// chip8.rs: chip8 in rust
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
use std::time::Duration;
use std::vec::Vec;

use log::{trace, warn};
use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};

use crate::{counter, fuse, input, vram};

#[derive(Debug)]
pub enum Chip8Message {
    ExecToggle,
    // Stops exec, keeps pc as is
    ExecPause,
    // Stops exec, sets pc to 0x200
    ExecStop,
    // Resume exec from wherever pc points
    ExecStart,
    // Stop exec, Load ROM, sets pc to 0x200
    LoadROM(Vec<u8>),
}

#[allow(non_snake_case)]
pub struct Chip8 {
    /*
     * 0x000-0x1FF - Space for the original Chip 8 interpreter and currently, fontsets
     * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
     * 0x200-0xFFF - Program binary and work RAM
     */
    memory: [u8; 4096],

    /*
     * CPU registers: The Chip 8 has 15 8-bit general purpose registers
     * named V0,V1 up to VE. The 16th register is used  for the ‘carry flag’.
     */
    vS: [u8; 16],

    // Index register
    i: u16, // u12

    // program counter
    pc: u16, // u12

    // Stack needed to jump to certain addresses or call subroutines
    // The chip8 stack has 16 levels
    sp: u8,

    running: bool,

    // 60hz counter channels
    sound_timer: counter::CounterHandle,
    delay_timer: counter::CounterHandle,

    // Keypad buttons, pressed or not
    input: input::InputHandle,

    // Video RAM, for SDL or other library to read from in a thread safe manner
    video: vram::VRAMHandle,

    // mbox for execution control
    exec: mpsc::Receiver<Chip8Message>,
}

impl Chip8 {
    pub fn new(
        input: input::InputHandle,
        video: vram::VRAMHandle,
        sound_timer: counter::CounterHandle,
        delay_timer: counter::CounterHandle,
        exec: mpsc::Receiver<Chip8Message>,
    ) -> Chip8 {
        Chip8 {
            memory: [0u8; 4096],
            vS: [0u8; 16],

            i: 0x50u16,
            pc: 0x200u16,
            sp: 0u8,

            running: false,

            delay_timer,
            sound_timer,
            input,
            video,
            exec,
        }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        self.load_bytes_at(rom, 0x200);
        self.reset_pc();
    }

    /// MAGIC NUMBER 3584:  0x1000 (4096/ram size) - 0x200 (self.pc / start of exec)
    pub fn load_bytes_at(&mut self, bytes: &Vec<u8>, at: usize) {
        let bl = bytes.len();
        let idx = match bl {
            _ if { bl < 3584 } => bl,
            _ => 3584,
        };

        self.memory[at..(at + idx)].copy_from_slice(&bytes[0..idx]);
    }

    pub fn reset_pc(&mut self) {
        self.pc = 0x200;
    }

    pub fn handle_message(&mut self, msg: Chip8Message) {
        match msg {
            Chip8Message::ExecPause => {
                self.running = false;
            }
            Chip8Message::ExecStop => {
                self.running = false;
                self.pc = 0x200;
            }
            Chip8Message::ExecStart => {
                self.running = true;
            }
            Chip8Message::ExecToggle => self.running = !self.running,
            Chip8Message::LoadROM(rom) => {
                self.load_rom(&rom);
            }
        }
    }

    pub async fn cycle(&mut self) {
        if self.running {
            // fetch
            if self.pc >= 0x1000
            // 4096
            {
                self.pc = 0x200;
            }
            let pc = self.pc as usize;
            let highbits: u16 = self.memory[pc] as u16;
            // Weird, emulator specific (I believe) quirk time
            // Wrap pc + 1 (byte) to 0x200
            let lowbits: u16 = if self.pc == 0x0FFF {
                self.memory[0x200] as u16
            } else {
                self.memory[(self.pc + 1) as usize] as u16
            };

            // Encode
            let mut opcode: u16 = highbits;
            opcode = (opcode << 8) | lowbits;

            // Decode/Execute
            trace!("PC[0x{:0>4X}]: 0x{:0>4X}", self.pc, opcode);
            match opcode {
                0x00E0 => self.video.clear_screen().await,
                0x00EE => self.ret(),
                0x1000..=0x1FFF => {
                    self.pc = (opcode & 0x0FFF) - 2;
                }
                0x2000..=0x2FFF => {
                    let addr = 0x0FFF & opcode;
                    let lowbits = (0x00FF & self.pc) as u8;
                    let highbits = ((0xFF00 & self.pc) >> 8) as u8;
                    self.sp += 2;
                    let sp = self.sp as usize;
                    self.memory[sp] = highbits;
                    self.memory[sp + 1] = lowbits;
                    self.pc = addr - 2;
                }
                0x3000..=0x3FFF => {
                    let x: u8 = ((0x0F00 & opcode) >> 8) as u8;
                    let kk: u8 = (0x00FF & opcode) as u8;
                    if self.vS[x as usize] == kk {
                        self.pc += 2;
                    }
                }
                0x4000..=0x4FFF => {
                    let x: u8 = ((0x0F00 & opcode) >> 8) as u8;
                    let kk: u8 = (0x00FF & opcode) as u8;
                    if self.vS[x as usize] != kk {
                        self.pc += 2;
                    }
                }
                0x5000..=0x5FFF => {
                    let ending = opcode & 0x000F;
                    if ending != 0x0 {
                        unknown_opcode(opcode);
                    } else {
                        let vx = ((0x0F00 & opcode) >> 8) as usize;
                        let vy = ((0x00F0 & opcode) >> 4) as usize;
                        if self.vS[vx] == self.vS[vy] {
                            self.pc += 2;
                        }
                    }
                }
                0x6000..=0x6FFF => {
                    let x: usize = (((0x0F00 & opcode) >> 8) as u8) as usize;
                    let kk: u8 = (0x00FF & opcode) as u8;
                    self.vS[x] = kk;
                }
                0x7000..=0x7FFF => {
                    let x: usize = (((0x0F00 & opcode) >> 8) as u8) as usize;
                    let kk: u8 = (0x00FF & opcode) as u8;
                    self.vS[x] = self.vS[x].wrapping_add(kk);
                }
                0x8000..=0x8FFF => {
                    let x: usize = (((0x0F00 & opcode) >> 8) as u8) as usize;
                    let y: usize = (((0x00F0 & opcode) >> 4) as u8) as usize;
                    let ending = 0x000F & opcode;
                    match ending {
                        0x0 => self.vS[x] = self.vS[y],
                        0x1 => self.vS[x] |= self.vS[y],
                        0x2 => self.vS[x] &= self.vS[y],
                        0x3 => self.vS[x] ^= self.vS[y],
                        0x4 => {
                            let x_val: u16 = self.vS[x] as u16;
                            let y_val: u16 = self.vS[y] as u16;
                            let res: u16 = x_val + y_val;
                            // this truncates, right?
                            let to_store = res as u8;
                            // Carry if overflow
                            if res > 255 {
                                self.vS[15] = 1;
                            }
                            self.vS[x] = to_store;
                        }
                        0x5 => {
                            let x_val: u16 = self.vS[x] as u16;
                            let y_val: u16 = self.vS[y] as u16;
                            if x_val > y_val {
                                self.vS[15] = 1;
                            }
                            self.vS[x] = self.vS[x].wrapping_sub(self.vS[y]);
                        }
                        0x6 => {
                            let y_val = self.vS[y];
                            let flag = 0b00000001 & y_val;
                            self.vS[15] = flag;
                            self.vS[x] = y_val >> 1;
                        }
                        0x7 => {
                            let x_val = self.vS[x];
                            let y_val = self.vS[y];
                            if x_val > y_val {
                                self.vS[15] = 0;
                            } else {
                                self.vS[15] = 1;
                            }
                            self.vS[x] = y_val.wrapping_sub(x_val);
                        }
                        0xE => {
                            let y_val = self.vS[y];
                            let msb = (0b10000000 & y_val).rotate_left(1);
                            self.vS[15] = msb;
                            self.vS[x] = y_val << 1;
                        }
                        _ => unknown_opcode(opcode),
                    }
                }
                0x9000..=0x9FFF => {
                    let ending = 0x000F & opcode;
                    let x = (0x0F00 & opcode) >> 8;
                    let y = (0x00F0 & opcode) >> 4;
                    match ending {
                        0x0 => {
                            let x_val = self.vS[x as usize];
                            let y_val = self.vS[y as usize];
                            if x_val != y_val {
                                self.pc += 2;
                            }
                        }
                        _ => unknown_opcode(opcode),
                    }
                }
                0xA000..=0xAFFF => {
                    self.i = opcode & 0x0FFF;
                }
                0xB000..=0xBFFF => {
                    self.pc = (0xFFF & opcode) + (self.vS[0] as u16);
                }
                0xC000..=0xCFFF => {
                    let x = (((0x0F00 & opcode) >> 8) as u8) as usize;
                    let nn = (0xFF & opcode) as u8;
                    let rand_byte: u8 = rand::random::<u8>();
                    let masked = rand_byte & nn;
                    self.vS[x] = masked;
                }
                0xD000..=0xDFFF => {
                    // Decode locations and values*
                    let vx = self.vS[((0xF00 & opcode) >> 8) as usize] as usize;
                    let vy = self.vS[((0x0F0 & opcode) >> 4) as usize] as usize;
                    let n = 0xF & (opcode as usize);
                    let mut sprite = Vec::with_capacity(n);
                    for i in 0..n {
                        sprite.push(self.memory[(self.i as usize + i)])
                    }

                    self.draw(vx, vy, &sprite).await
                }
                0xE000..=0xEFFF => {
                    // Register where keycode is stored
                    let x = (0x0F00 & opcode) >> 8;
                    let nn = 0x00FF & opcode;

                    match nn {
                        0x9E => {
                            // Keycode itself, should be between 0-F
                            let key = self.vS[x as usize];

                            // if keycode is pressed
                            if self.input.pressed(key).await {
                                self.pc += 2;
                            }
                        }
                        0xA1 => {
                            // Keycode itself, should be between 0-F
                            let key = self.vS[x as usize];

                            // if keycode is pressed
                            if !self.input.pressed(key).await {
                                self.pc += 2;
                            }
                        }
                        _ => unknown_opcode(opcode),
                    }
                }
                0xF000..=0xFFFF => {
                    let x = ((0x0F00 & opcode) >> 8) as usize;
                    let nn = 0x00FF & opcode;
                    match nn {
                        0x7 => {
                            self.vS[x] = self.delay_timer.get().await;
                        }
                        0xA => {
                            todo!("Waiting for input");
                        }
                        0x15 => {
                            self.delay_timer.set(self.vS[x]).await;
                        }
                        0x18 => {
                            self.sound_timer.set(self.vS[x]).await;
                        }
                        0x1E => self.i += self.vS[x] as u16,
                        0x29 => self.i = 0x50 + 5 * (self.vS[x] as u16),
                        0x33 => {
                            let value = self.vS[x];
                            let ones = value % 10;
                            let tens = (value / 10) % 10;
                            let huns = value / 100;
                            self.memory[self.i as usize] = huns;
                            self.memory[self.i as usize + 1] = tens;
                            self.memory[self.i as usize + 2] = ones;
                        }
                        0x55 => {
                            for idx in 0..=x {
                                let ix = (self.i + (idx as u16)) as usize;
                                self.memory[ix] = self.vS[idx];
                            }
                        }
                        0x65 => {
                            for idx in 0..=x {
                                let ix = (self.i + (idx as u16)) as usize;
                                self.vS[idx] = self.memory[ix];
                            }
                            self.i += x as u16 + 1;
                        }
                        _ => unknown_opcode(opcode),
                    }
                }
                _ => unknown_opcode(opcode),
            }

            // Increment the program counter
            self.pc += 2;
        }
    }

    fn sp_addr(&self) -> u16 {
        let sp: usize = self.sp as usize;
        let highbits: u8 = self.memory[sp];
        let lowbits: u8 = self.memory[sp + 1];
        let mut address: u16 = highbits as u16;
        address <<= 8;
        address |= lowbits as u16;

        address
    }

    fn ret(&mut self) {
        // Sets PC to the address at the top of the stack,
        // then subtracts a u16 from the stack pointer
        self.pc = self.sp_addr();
        self.sp -= 2;
    }

    async fn draw(&mut self, vx: usize, vy: usize, bytes: &[u8]) {
        if !self.running {
            return;
        }
        let (sx, sy) = self.video.get_screen_size();
        let tx = vx % sx;
        let ty = vy % sy;
        let mut collision: u8 = 0;

        let masks: [u8; 8] = [
            0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010,
            0b00000001,
        ];

        for (row, b) in bytes.iter().enumerate() {
            let y = (row + ty) % sy;
            if y < sy {
                for (col, mask) in masks.iter().enumerate() {
                    let x = (tx + col) % sx;
                    let bmask = mask & b;
                    let cur_value = self.video.get_pixel(x, y).await;
                    if bmask > 0 {
                        if cur_value {
                            self.video.set_pixel(x, y, false).await;
                            collision = 1;
                        } else {
                            self.video.set_pixel(x, y, true).await;
                        }
                    }
                    if (x + 1) == sx {
                        break;
                    }
                }
                if (y + 1) == sy {
                    break;
                }
            }
        }

        self.vS[15] = collision;
    }
}

fn unknown_opcode(opcode: u16) {
    warn!("Unknown opcode: 0x{:0<4X}", opcode);
}

pub struct Chip8Handle {
    pub sound_timer: counter::CounterHandle,
    pub delay_timer: counter::CounterHandle,
    pub send: mpsc::Sender<Chip8Message>,
    pub running: bool,
}

impl Chip8Handle {
    pub fn new(
        freq: f64,
        rom: Option<Vec<u8>>,
        input: input::InputHandle,
        video: vram::VRAMHandle,
        fuse: fuse::FuseHandle,
    ) -> Self {
        let sound_timer = counter::CounterHandle::new();
        let delay_timer = counter::CounterHandle::new();
        let (send, recv) = mpsc::channel(10);
        let c8 = init_chip8(
            &rom,
            input,
            video,
            sound_timer.clone(),
            delay_timer.clone(),
            recv,
        );
        tokio::spawn(async move { run_chip8(freq, fuse, c8).await });

        Self {
            sound_timer,
            delay_timer,
            send,
            running: false,
        }
    }

    pub async fn load_rom(&self, rom: Vec<u8>) {
        let msg = Chip8Message::LoadROM(rom);
        self.send.send(msg).await.unwrap();
    }

    pub async fn toggle_exec(&self) {
        self.send.send(Chip8Message::ExecToggle).await.unwrap();
    }

    pub async fn pause(&self) {
        self.send.send(Chip8Message::ExecPause).await.unwrap();
    }

    pub async fn unpause(&self) {
        self.send.send(Chip8Message::ExecStart).await.unwrap();
    }
}

pub fn init_chip8(
    rom: &Option<Vec<u8>>,
    input: input::InputHandle,
    video: vram::VRAMHandle,
    sound: counter::CounterHandle,
    delay: counter::CounterHandle,
    exec: mpsc::Receiver<Chip8Message>,
) -> Chip8 {
    let mut vm = Chip8::new(input, video, sound, delay, exec);

    // Fontset
    let fontset = vec![
        0xF0u8, 0x90, 0x90, 0x90, 0xF0, // 0 | 0x50
        0x20, 0x60, 0x20, 0x20, 0x70, // 1 | 0x55
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2 | 0x5A
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3 | 0x5F
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4 | 0x64
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5 | 0x69
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6 | 0x6E
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7 | 0x73
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8 | 0x78
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9 | 0x7D
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A | 0x82
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B | 0x87
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C | 0x8C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D | 0x91
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E | 0x96
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F | 0x9B
        0x60, 0x80, 0xF0, 0x10, 0x60, // S | 0xA0
    ];
    vm.load_bytes_at(&fontset, 0x50);

    match rom {
        Some(x) => {
            vm.load_rom(x);
        }
        None => {}
    }

    // vm.memory[0x3] = 0xA;
    vm
}

async fn run_chip8(frequency: f64, fuse: fuse::FuseHandle, mut c8: Chip8) {
    trace!("Start Chip8 Task");
    let mut ival = interval(Duration::from_secs_f64(frequency));
    ival.set_missed_tick_behavior(MissedTickBehavior::Skip);
    while fuse.alive() {
        ival.tick().await;
        c8.cycle().await;

        if let Ok(msg) = c8.exec.try_recv() {
            c8.handle_message(msg)
        }
    }
    trace!("Exiting Chip8 Task");
}
