const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Emulator{
    pc: u16,
    sp: u8,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; 16],
    i_register: u16,
    stack : [u16; 16],
    sound_timer: u8,
    delay_timer: u8,
}