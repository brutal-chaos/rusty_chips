/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::default::Default;

pub struct Chip8 {
    // Clear Display (SDL - black screen)
    pub cls: bool,

    /*
     * 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
     * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
     * 0x200-0xFFF - Program ROM and work RAM
     */
    pub memory: [u8; 4096],

    /*
     * CPU registers: The Chip 8 has 15 8-bit general purpose registers
     * named V0,V1 up to VE. The 16th register is used  for the ‘carry flag’.
     */
    pub registers: [u8; 16],

    // Index register
    pub i: u16,

    // program counter
    pub pc: u16,

    // Screen - 64 x 32 pixels
    pub graphics: [u8; 64 * 32],

    // No timers, but 60hz counters
    pub delay_timer: u8,
    pub sound_timer: u8,

    // Stack needed to jump to certain addresses or call subroutines
    // The chip8 stack has 16 levels
    pub stack_pointer: u16,
    pub stack: [u16; 16],

    // Chip8 keypad had 16 keys - keep track of the kb state
    pub keypad: [u8; 16],
}

impl Default for Chip8 {
    fn default() -> Chip8 {
        Chip8 {
            // Let's clear the screen at start
            cls: true,
            memory: [0u8; 4096],
            registers: [0u8; 16],
            i: 0u16,
            pc: 0x200u16,
            graphics: [0u8; 64 * 32],
            delay_timer: 0u8,
            sound_timer: 0u8,
            stack_pointer: 0u16,
            stack: [0u16; 16],
            keypad: [0u8; 16],
        }
    }
}

impl Chip8 {
    pub fn cycle(&mut self) {
        // fetch
        let highbits: u8 = self.memory[self.pc as usize];
        let lowbits: u8 = self.memory[(self.pc + 1) as usize];
        let mut opcode: u16 = highbits as u16;
        opcode = (opcode << 8) | lowbits as u16;

        // Decode
        match opcode {
            // Clear screen
            0x00e0 => self.cls = true,
            // RET
            0x00ee => {
                // Move program counter to the top of the stack
                self.pc = self.stack[(self.stack_pointer as usize)];
                // Decrement the stack pointer
                self.stack_pointer -= 1
            }
            // 1nnn - JP addr - jump to nnn
            op @ 0x1000..=0x1fff => self.pc = op ^ 0x1000,
            // 2nnn - CALL addr - call subroutine at nnn
            op @ 0x2000..=0x2fff => {
                let nnn = op ^ 0x2000;
                // Increment the stack pointer
                self.stack_pointer += 1;
                // Put the value of pc ontop of the stack
                self.stack[self.stack_pointer as usize] = self.pc;
                // Set the pc to nnn
                self.pc = nnn;
            }
            // 3xkk - SE Vx, byte - Skip next instruction if Vx == kk
            op @ 0x3000..=0x3fff => {
                let V: u8 = ((op & 0x0F00) >> 8) as u8;
                let byte: u8 = (op & 0x00FF) as u8;
                if self.registers[V as usize] == byte {
                    self.pc += 2;
                }
            }
            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            op @ 0x4000..=0x4fff => {
                let V: u8 = ((op & 0x0F00) >> 8) as u8;
                let byte: u8 = (op & 0x00FF) as u8;
                if self.registers[V as usize] != byte {
                    self.pc += 2;
                }
            }
            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx == Vy
            op @ 0x5000..=0x5ff0 if op % 16 == 0 => {
                let vx: u8 = ((op & 0x0f00) >> 8) as u8;
                let vy: u8 = ((op & 0x00f0) >> 4) as u8;
                if vx == vy {
                    self.pc += 2;
                }
            }
            // 6xkk - LD Vx, byte -  Set Vx = kk
            op @ 0x6000..=0x6fff => {
                self.registers[((op & 0x0f00) >> 8) as usize] = (op & 0x00ff) as u8;
            }
            // 7xkk - ADD Vx, byte -  Set Vx = Vx + kk
            op @ 0x7000..=0x7fff => {
                self.registers[((op & 0x0f00) >> 8) as usize] += (op & 0x00ff) as u8;
            }
            // 8xy0 - 8xye
            op @ 0x8000..=0x8ffe => {
                let x = (op & 0x000f) as usize;
                let vy = ((op & 0x00f0) >> 4) as usize;
                let vx = ((op & 0x0f00) >> 8) as usize;

                match x {
                    // 8xy0 - LD Vx, Vy - Set Vx = Vy.
                    0 => self.registers[vx] = self.registers[vy],
                    // 8xy1 - OR Vx, Vy - Set Vx = Vx OR Vy
                    1 => self.registers[vx] = self.registers[vx] | self.registers[vy],
                    // 8xy2 - AND Vx, Vy - Set Vx = Vx AND Vy
                    2 => self.registers[vx] = self.registers[vx] & self.registers[vy],
                    // 8xy3 - XOR Vx, Vy - Set Vx = Vx XOR Vy
                    3 => self.registers[vx] = self.registers[vx] ^ self.registers[vy],
                    // 8xy4 - ADD Vx, Vy - Set Vx = Vx + Vy, set VF = carry
                    4 => {
                        let added = self.registers[vx] + self.registers[vy];
                        if added > 255 {
                            self.registers[vx] = 0xff;
                            self.registers[15] = 1;
                        } else {
                            self.registers[vx] = added;
                            self.registers[15] = 0;
                        }
                    }
                    // 8xy5 - SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow
                    5 => {
                        if self.registers[vx] > self.registers[vy] {
                            self.registers[15] = 1;
                        } else {
                            self.registers[15] = 0;
                        }
                        self.registers[vx] -= self.registers[vy];
                    }
                    // 8xy6 - SHR Vx {, Vy} - Set Vx = Vx SHR 1
                    6 => {
                        if self.registers[vx] & 1 == 1 {
                            self.registers[15] = 1;
                        } else {
                            self.registers[15] = 0;
                        }
                        self.registers[vx] /= 2;
                    }
                    // 8xy7 - SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow
                    7 => {
                        if self.registers[vy] > self.registers[vx] {
                            self.registers[15] = 1;
                        } else {
                            self.registers[15] = 0;
                        }
                        self.registers[vx] -= self.registers[vy];
                    }
                    // 8xyE - SHL Vx {, Vy} - Set Vx = Vx SHL 1
                    14 => {
                        if self.registers[vx] > 127 {
                            self.registers[15] = 1;
                        } else {
                            self.registers[15] = 0;
                        }
                        self.registers[vx] *= 2;
                    }
                    _ => println! {"Unknown opcode: 0x{:X}", op},
                }
            }
            // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            op @ 0x9000..=0x9ff0 if op % 16 == 0 => {
                let vy: usize = ((op & 0x00f0) >> 4) as usize;
                let vx: usize = ((op & 0x0f00) >> 8) as usize;
                if self.registers[vx] != self.registers[vy] {
                    self.pc += 2;
                }
            }
            // Unsupported
            op @ _ => println!("Unknown opcode: 0x{:X}", op),
        }
        // Execute
        // Increment the counter
        self.pc += 2;
        // Update timers
    }
}

pub(crate) fn init_chip8() -> Chip8 {
    let mut vm = Chip8 {
        ..Default::default()
    };

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
        vm.memory[c] = fontset[c];
    }

    vm
}
