use std::default;

use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDRESS: u16 = 0x200;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,

    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],

    dt: u8,
    st: u8,
}

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

impl Emu {
    pub fn new() -> Self {
        let mut emu = Emu {
            pc: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();
        // decode and execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }
            (0x1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (0x2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            (0x3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            (0x4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0x6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }
            (0x7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            (0x8, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            (0x8, _, _, 0x1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            (0x8, _, _, 0x2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            (0x8, _, _, 0x3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            (0x8, _, _, 0x4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (result, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_reg[x] = result;
                self.v_reg[0xF] = new_vf;
            }
            (0x8, _, _, 0x5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (result, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = result;
                self.v_reg[0xF] = new_vf;
            }
            (0x8, _, _, 0x6) => {
                let x = digit2 as usize;
                let bit = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = bit;
            }
            (0x8, _, _, 0x7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (result, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = result;
                self.v_reg[0xF] = new_vf;
            }
            (0x8, _, _, 0xE) => {
                let x = digit2 as usize;
                let bit = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = bit;
            }
            (0x9, _, _, 0x0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = self.v_reg[0] as u16 + nnn;
            }
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            (0xD, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let n = digit4;
                let x_coord = self.v_reg[x] as u16;
                let y_coord = self.v_reg[y] as u16;

                let mut flipped = false;
                for row in 0..n {
                    let address = self.i_reg + row as u16;
                    let pixels = self.ram[address as usize];
                    for col in 0..8 {
                        if (pixels & (0b1000_0000 >> col)) != 0 {
                            let x = (x_coord + col) as usize % SCREEN_WIDTH;
                            let y = (y_coord + row) as usize % SCREEN_HEIGHT;

                            let idx = x + y * SCREEN_WIDTH;

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                let new_vf = if flipped { 1 } else { 0 };
                self.v_reg[0xF] = new_vf;
            }
            (0xE, _, 0x9, 0xE) => {
                let x = digit2 as usize;
                let val_x = self.v_reg[x] as usize;
                let key = self.keys[val_x];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                let x = digit2 as usize;
                let val_x = self.v_reg[x] as usize;
                let key = self.keys[val_x];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }
            (0xF, _, 0x0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for (i, key) in self.keys.iter().enumerate() {
                    if *key {
                        pressed = true;
                        self.v_reg[x] = i as u8;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }
            (0xF, _, 0x1, 0x8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }
            (0xF, _, 0x1, 0xE) => {
                let x = digit2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            }
            (0xF, _, 0x2, 0x9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }
            (0xF, _, 0x3, 0x3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0).floor() as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 0x5, 0x5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }
            default => unimplemented!("We didn't implement all the instructions yet"),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
            if self.st == 0 {
                // beep
            }
        }
    }

    // frontend helpers
    pub fn get_display(&self) -> &[bool] {
        return &self.screen;
    }

    pub fn key_press(&mut self, idx: usize, pressed: bool) {
        if idx < NUM_KEYS {
            return;
        }
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start_address = START_ADDRESS as usize;
        let end_address = START_ADDRESS as usize + data.len();
        self.ram[start_address..end_address].copy_from_slice(data);
    }
}
