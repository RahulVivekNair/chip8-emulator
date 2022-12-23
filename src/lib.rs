const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const START_ADDR: u16 = 0x200;


pub struct Emulator {
    pc: u16,
    sp: u8,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; 16],
    i_register: u16,
    stack: [u16; 16],
    sound_timer: u8,
    delay_timer: u8,
    keys: [bool; 16],
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; 16],
            i_register: 0,
            sp: 0,
            stack: [0; 16],
            sound_timer: 0,
            delay_timer: 0,
            keys: [false; 16],
        }
    }
}
