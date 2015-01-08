use std::default::Default;

pub struct Chip8 {
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
    pub graphics: [u8; 64*32],

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
    #[inline]
    fn default() -> Chip8 {
        Chip8 {
            memory: [0u8; 4096],

            registers: [0u8; 16],

            i: 0u16,

            pc: 0x200u16,

            graphics: [0u8; 64*32],

            delay_timer: 0u8,
            sound_timer: 0u8,

            stack_pointer: 0u16,
            stack: [0u16; 16],

            keypad: [0u8; 16],
        }
    }
}

impl Chip8 {
   fn cycle(&self) {
        // fetch
        // Decode
        // Execute
        // Update timers
    }
}
