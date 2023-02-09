/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{interval, MissedTickBehavior};

#[allow(non_snake_case)]
pub struct Chip8 {
    /*
     * 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
     * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
     * 0x200-0xFFF - Program ROM and work RAM
     */
    pub memory: [u8; 4096],

    pub vram: [[bool; 64]; 32],

    /*
     * CPU registers: The Chip 8 has 15 8-bit general purpose registers
     * named V0,V1 up to VE. The 16th register is used  for the ‘carry flag’.
     */
    pub vS: [u8; 16],

    // Index register
    pub i: u16, // u12

    // program counter
    pub pc: u16, // u12

    // 60hz counter channels
    delay_tx: Option<tokio::sync::watch::Sender<u8>>,
    delay_rx: Option<tokio::sync::watch::Receiver<u8>>,
    sound_tx: Option<tokio::sync::watch::Sender<u8>>,
    sound_rx: Option<tokio::sync::watch::Receiver<u8>>,

    // Stack needed to jump to certain addresses or call subroutines
    // The chip8 stack has 16 levels
    pub sp: u8,

    // Screen Details
    pub screen_width: usize,
    pub screen_height: usize,

    // Keypad buttons, pressed or not
    keypad: [bool; 16],

    // Comms channels
    input: Option<watch::Receiver<char>>,
    video: Option<watch::Sender<[[bool; 64]; 32]>>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0u8; 4096],
            vram: [[false; 64]; 32],
            vS: [0u8; 16],

            i: 0x50u16,
            pc: 0x200u16,
            delay_tx: None,
            delay_rx: None,
            sound_tx: None,
            sound_rx: None,
            sp: 0u8,

            screen_height: 64,
            screen_width: 32,
            keypad: [false; 16],

            input: None,
            video: None,
        }
    }

    pub fn cycle(&mut self) {
        // fetch
        if self.pc >= 4096 {
            self.pc = 0;
        }
        let pc = self.pc as usize;
        let highbits: u8 = self.memory[pc];
        let lowbits: u8 = self.memory[(pc + 1)];

        // Encode
        let mut opcode: u16 = highbits as u16;
        opcode = (opcode << 8) | lowbits as u16;

        // Decode/Execute
        println!("Opcode: {:X}", opcode);
        match opcode {
            0x00E0 => {
                for y in 0..32 {
                    for x in 0..64 {
                        self.vram[y][x] = false;
                    }
                }
            }
            0x00EE => self.ret(),
            0x1000..=0x1FFF => self.pc = 0x0FFF & opcode,
            0x2000..=0x2FFF => {
                let addr = 0x0FFF & opcode;
                let lowbits = (0x00FF & self.pc) as u8;
                let highbits = ((0xFF00 & self.pc) >> 8) as u8;
                self.sp += 2;
                let sp = self.sp as usize;
                self.memory[sp] = highbits;
                self.memory[sp + 1] = lowbits;
                self.pc = addr;
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
                    let vx = ((0x0F00 >> 8) as u8) as usize;
                    let vy = ((0x00F0 >> 4) as u8) as usize;
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
                self.i = 0xFFF & opcode;
            }
            0xB000..=0xBFFF => {
                self.pc = (0x0FFF & opcode) + (self.vS[0] as u16);
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
                let vx = self.vS[(((0x0F00 & opcode) >> 8) as u8) as usize];
                let vy = self.vS[(((0x00F0 & opcode) >> 4) as u8) as usize];
                let n = (0x000F & opcode) as u8;
                let mut sprite = Vec::with_capacity(n as usize);
                for i in 0..n {
                    sprite.push(self.memory[(self.i + i as u16) as usize])
                }

                self.draw(vx, vy, &sprite);
                if self.video.is_some() {
                    self.video.as_ref().unwrap().send(self.vram).unwrap();
                    println!("Sending video bools");
                }
            }
            0xE000..=0xEFFF => {
                // Register where keycode is stored
                let x = (0x0F00 & opcode) >> 8;
                let nn = 0x00FF & opcode;

                match nn {
                    0x9E => {
                        // Keycode itself, should be between 0-F
                        let key = self.vS[x as usize] as usize;

                        // if keycode is pressed
                        if self.keypad[key] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // Keycode itself, should be between 0-F
                        let key = self.vS[x as usize] as usize;

                        // if keycode is pressed
                        if !self.keypad[key] {
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
                        if self.delay_rx.is_some() {
                            self.vS[x] = *self.delay_rx.as_ref().unwrap().borrow();
                        }
                    }
                    0xA => {
                        println!("Waiting for input");
                        if self.input.is_some() {
                            let ipt = self.input.as_ref().expect("dafuq?");

                            let mut latest: char = *ipt.borrow();
                            while latest == (0u8 as char) {
                                latest = *ipt.borrow();
                            }
                            self.keypad[latest as usize] = true;
                        }
                    }
                    0x15 => {
                        if self.delay_tx.is_some() {
                            self.delay_tx.as_ref().unwrap().send(self.vS[x]).unwrap();
                        }
                    }
                    0x18 => {
                        if self.sound_tx.is_some() {
                            self.sound_tx.as_ref().unwrap().send(self.vS[x]).unwrap();
                        }
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

    fn sp_addr(self: &Self) -> u16 {
        let sp: usize = self.sp as usize;
        let highbits: u8 = self.memory[sp];
        let lowbits: u8 = self.memory[sp + 1];
        let mut address: u16 = highbits as u16;
        address <<= 8;
        address |= lowbits as u16;
        let result = address;

        result
    }

    fn ret(self: &mut Self) {
        // Sets PC to the address at the top of the stack, then subtracts 1 from the stack
        // pointer
        self.pc = self.sp_addr();
        self.sp -= 2;
    }

    fn draw(self: &mut Self, vx: u8, vy: u8, bytes: &Vec<u8>) {
        let tx = (vx % 64) as usize;
        let ty = (vy % 32) as usize;

        let masks: [u8; 8] = [
            0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010,
            0b00000001,
        ];

        for (ix, b) in bytes.iter().enumerate() {
            let y = ix + ty;
            for (z, mask) in masks.iter().enumerate() {
                let x = tx + z;
                if x < 64 && y < 32 {
                    let cur_value = self.vram[y][x];
                    let new_value = cur_value ^ ((mask & b) >= 1);
                    if cur_value != new_value {
                        self.vS[15] = 1;
                    }
                    self.vram[y][x] = new_value;
                }
            }
        }
    }
}

fn unknown_opcode(opcode: u16) {
    println!("Unknown opcode: {:X}", opcode);
}

pub(crate) fn init_chip8(
    rom: Option<Vec<u8>>,
    input: watch::Receiver<char>,
    video: watch::Sender<[[bool; 64]; 32]>,
    vdclr: watch::Sender<bool>,
) -> Chip8 {
    let mut vm = Chip8::new();
    vm.input = Some(input);
    vm.video = Some(video);
    vm.vdclr = Some(vdclr);

    // Fontset
    let fontset = [
        0xF0u8, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    for c in 0..fontset.len() {
        vm.memory[0x50 + c] = fontset[c];
    }

    match rom {
        Some(x) => {
            for d in 0..x.len() {
                vm.memory[0x200 + d] = x[d];
            }
        }
        None => {}
    }

    vm
}

#[allow(dead_code)]
pub async fn chip8_dec_timer(
    alive: tokio::sync::watch::Receiver<bool>,
    outval: tokio::sync::watch::Sender<u8>,
    inval: tokio::sync::watch::Receiver<u8>,
) {
    println!("Start Dec Timer Task");
    let mut ival = interval(Duration::from_secs_f64(0.01667));
    ival.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ival.tick().await;
    while *alive.borrow() {
        ival.tick().await;
        let cur = *inval.borrow();
        if cur > 0 {
            outval.send(cur - 1).unwrap_or(());
        }
    }
    println!("Exiting Dec Timer Task");
}

pub async fn chip8_runner(
    alive: tokio::sync::watch::Receiver<bool>,
    input: tokio::sync::watch::Receiver<char>,
    video: tokio::sync::watch::Sender<[[bool; 64]; 32]>,
    vdclr: tokio::sync::watch::Sender<bool>,
    rom: Option<Vec<u8>>,
) {
    println!("Start chip8_runner Task");
    let mut chip = init_chip8(rom, input, video, vdclr);

    let (delay_tx_chip, delay_rx_timer) = watch::channel(0);
    let (delay_tx_timer, delay_rx_chip) = watch::channel(0);
    let (sound_tx_chip, sound_rx_timer) = watch::channel(0);
    let (sound_tx_timer, sound_rx_chip) = watch::channel(0);
    let delay_alive = alive.clone();
    let delay_timer =
        tokio::spawn(
            async move { chip8_dec_timer(delay_alive, delay_tx_timer, delay_rx_timer).await },
        );
    let sound_alive = alive.clone();
    let sound_timer =
        tokio::spawn(
            async move { chip8_dec_timer(sound_alive, sound_tx_timer, sound_rx_timer).await },
        );
    chip.delay_tx = Some(delay_tx_chip);
    chip.delay_rx = Some(delay_rx_chip);
    chip.sound_tx = Some(sound_tx_chip);
    chip.sound_rx = Some(sound_rx_chip);
    let runner_alive = alive.clone();
    let chip_clock = tokio::spawn(async move { runner(runner_alive, &mut chip).await });
    let _ = tokio::join!(delay_timer, sound_timer, chip_clock);
    println!("Exiting chip8_runner Task");
}

async fn runner(alive: tokio::sync::watch::Receiver<bool>, chip: &mut Chip8) {
    println!("Start Runner Task");
    let mut ival = interval(Duration::from_secs_f64(2.083e-8));
    ival.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut counter = 100;
    while *alive.borrow()
    /* && counter > 0 */
    {
        ival.tick().await;
        chip.cycle();
        //counter -= 1;
    }
    println!("Exiting Runner Task");
}
