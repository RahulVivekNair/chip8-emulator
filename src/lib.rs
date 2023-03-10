const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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

pub struct Emulator {
    pc: u16,
    sp: u16,
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
        let mut new_emu = Self {
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
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; 16];
        self.i_register = 0;
        self.sp = 0;
        self.stack = [0; 16];
        self.sound_timer = 0;
        self.delay_timer = 0;
        self.keys = [false; 16];
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();

        // Decode
        // Execute
        self.execute(op);
    }
    fn execute(&mut self, op: u16) {
        let h_digit1 = (op & 0xF000) >> 12;
        let h_digit2 = (op & 0x0F00) >> 8;
        let h_digit3 = (op & 0x00F0) >> 4;
        let h_digit4 = op & 0x000F;

        match (h_digit1, h_digit2, h_digit3, h_digit4) {
            //NOP instruction
            (0, 0, 0, 0) => return,
            //Clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            //Return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            //Jump to address
            (1, _, _, _) => {
                self.pc = op & 0x0FFF;
            }
            //Call subroutine
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            }
            //skip next instruction
            (3, _, _, _) => {
                let reg = h_digit2;
                let val = op & 0x00FF;
                if self.v_registers[reg as usize] == val as u8 {
                    self.pc += 2;
                }
            }
            //skip next instruction if not equal
            (4, _, _, _) => {
                let reg = h_digit2;
                let val = op & 0x00FF;
                if self.v_registers[reg as usize] != val as u8 {
                    self.pc += 2;
                }
            }
            //skip next instruction if reg values are equal
            (5, _, _, _) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                if self.v_registers[reg1] == self.v_registers[reg2] {
                    self.pc += 2;
                }
            }
            //set register to value
            (6, _, _, _) => {
                let reg = h_digit2 as usize;
                let val = op & 0x00FF;
                self.v_registers[reg] = val as u8;
            }
            //add value to register
            (7, _, _, _) => {
                let reg = h_digit2 as usize;
                let val = op & 0x00FF;
                //wrapping add to avoid overflowss
                self.v_registers[reg] = self.v_registers[reg].wrapping_add(val as u8);
            }
            //set register to value of another register
            (8, _, _, 0) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                self.v_registers[reg1] = self.v_registers[reg2];
            }
            //bitwise or
            (8, _, _, 1) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                self.v_registers[reg1] |= self.v_registers[reg2];
            }
            //bitwise and
            (8, _, _, 2) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                self.v_registers[reg1] &= self.v_registers[reg2];
            }
            //bitwise xor
            (8, _, _, 3) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                self.v_registers[reg1] ^= self.v_registers[reg2];
            }
            //add register to another register
            (8, _, _, 4) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                let (val, overflow) =
                    self.v_registers[reg1].overflowing_add(self.v_registers[reg2]);
                self.v_registers[reg1] = val;
                self.v_registers[0xF] = if overflow { 1 } else { 0 };
            }
            //subtract register from another register
            (8, _, _, 5) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                let (val, overflow) =
                    self.v_registers[reg1].overflowing_sub(self.v_registers[reg2]);
                self.v_registers[reg1] = val;
                self.v_registers[0xF] = if overflow { 0 } else { 1 };
            }
            //shift right and store dropped bit in VF
            (8, _, _, 6) => {
                let reg1 = h_digit2 as usize;
                self.v_registers[0xF] = self.v_registers[reg1] & 0x1;
                self.v_registers[reg1] >>= 1;
            }
            //subtract register from another register
            (8, _, _, 7) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                let (val, overflow) =
                    self.v_registers[reg2].overflowing_sub(self.v_registers[reg1]);
                self.v_registers[reg1] = val;
                self.v_registers[0xF] = if overflow { 0 } else { 1 };
            }
            //shift left and store dropped bit in VF
            (8, _, _, 0xE) => {
                let reg1 = h_digit2 as usize;
                self.v_registers[0xF] = self.v_registers[reg1] >> 7;
                self.v_registers[reg1] <<= 1;
            }
            //skip next instruction if reg values are not equal
            (9, _, _, 0) => {
                let reg1 = h_digit2 as usize;
                let reg2 = h_digit3 as usize;
                if self.v_registers[reg1] != self.v_registers[reg2] {
                    self.pc += 2;
                }
            }
            //set I to value
            (0xA, _, _, _) => {
                let val = op & 0x0FFF;
                self.i_register = val;
            }
            //jump to address + value of register 0
            (0xB, _, _, _) => {
                let val = op & 0x0FFF;
                self.pc = val + self.v_registers[0] as u16;
            }
            //set register to random number and value
            (0xC, _, _, _) => {
                let reg = h_digit2 as usize;
                let val = op & 0x00FF;
                let rand = fastrand::u8(..);
                self.v_registers[reg] = rand & val as u8;
            }
            //draw sprite
            (0xD, _, _, _) => {
                let x_coord = self.v_registers[h_digit2 as usize] as usize;
                let y_coord = self.v_registers[h_digit3 as usize] as usize;
                let height = h_digit4 as usize;
                let mut flipped = false;
                for y_line in 0..height {
                    let pixels = self.ram[(self.i_register + y_line as u16) as usize];
                    for x_line in 0..8 as usize {
                        if (pixels & (0x80 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }
            }
            //skip next instruction if key in register is pressed
            (0xE, _, 9, 0xE) => {
                let reg = h_digit2 as usize;
                let key = self.v_registers[reg];
                if self.keys[key as usize] {
                    self.pc += 2;
                }
            }
            //skip next instruction if key in register is not pressed
            (0xE, _, 0xA, 1) => {
                let reg = h_digit2 as usize;
                let key = self.v_registers[reg];
                if !self.keys[key as usize] {
                    self.pc += 2;
                }
            }
            //set register to delay timer
            (0xF, _, 0, 7) => {
                let reg = h_digit2 as usize;
                self.v_registers[reg] = self.delay_timer;
            }
            //wait for keypress and store in register
            (0xF, _, 0, 0xA) => {
                let reg = h_digit2 as usize;
                let mut key_pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[reg] = i as u8;
                        key_pressed = true;
                    }
                }
                if !key_pressed {
                    self.pc -= 2;
                }
            }
            //set delay timer to register
            (0xF, _, 1, 5) => {
                let reg = h_digit2 as usize;
                self.delay_timer = self.v_registers[reg];
            }
            //set sound timer to register
            (0xF, _, 1, 8) => {
                let reg = h_digit2 as usize;
                self.sound_timer = self.v_registers[reg];
            }
            //add register to I
            (0xF, _, 1, 0xE) => {
                let reg = h_digit2 as usize;
                self.i_register = self.i_register.wrapping_add(self.v_registers[reg] as u16);
            }
            //set I to location of sprite for digit in register
            (0xF, _, 2, 9) => {
                let reg = h_digit2 as usize;
                self.i_register = (self.v_registers[reg] * 5) as u16;
            }
            //store BCD representation of register in memory
            (0xF, _, 3, 3) => {
                let reg = h_digit2 as usize;
                let val = self.v_registers[reg];
                self.ram[self.i_register as usize] = val / 100;
                self.ram[(self.i_register + 1) as usize] = (val / 10) % 10;
                self.ram[(self.i_register + 2) as usize] = (val % 100) % 10;
            }
            //store registers in memory
            (0xF, _, 5, 5) => {
                let reg = h_digit2 as usize;
                for i in 0..=reg {
                    self.ram[(self.i_register + i as u16) as usize] = self.v_registers[i];
                }
            }
            //read registers from memory
            (0xF, _, 6, 5) => {
                let reg = h_digit2 as usize;
                for i in 0..=reg {
                    self.v_registers[i] = self.ram[(self.i_register + i as u16) as usize];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:X}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }
            self.sound_timer -= 1;
        }
    }
}
