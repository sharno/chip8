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
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // true = pixel on, false = pixel off
    v_reg: [u8; NUM_REGS],
    i_reg: u16,

    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],

    dt: u8, // Delay Timer
    st: u8, // Sound Timer
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

        // Load fontset into lower memory
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
        let digit2 = (op & 0x0F00) >> 8; // Often the X register index
        let digit3 = (op & 0x00F0) >> 4; // Often the Y register index
        let digit4 = op & 0x000F; // Often the N value

        let nnn = op & 0x0FFF; // 12-bit address
        let nn = (op & 0x00FF) as u8; // 8-bit constant
        let x = digit2 as usize; // Index for V registers
        let y = digit3 as usize; // Index for V registers
        let n = digit4 as usize; // 4-bit height / value

        match (digit1, digit2, digit3, digit4) {
            // NOP (0000) - Often ignored, returning is fine
            (0, 0, 0, 0) => return,
            // CLS (00E0) - Clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // RET (00EE) - Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }
            // JP addr (1NNN) - Jump to address NNN
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            // CALL addr (2NNN) - Call subroutine at NNN
            (0x2, _, _, _) => {
                self.push(self.pc); // Push current PC (which points *after* this instruction)
                self.pc = nnn;
            }
            // SE Vx, byte (3XNN) - Skip next instruction if Vx == NN
            (0x3, _, _, _) => {
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte (4XNN) - Skip next instruction if Vx != NN
            (0x4, _, _, _) => {
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy (5XY0) - Skip next instruction if Vx == Vy
            (0x5, _, _, 0x0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte (6XNN) - Set Vx = NN
            (0x6, _, _, _) => {
                self.v_reg[x] = nn;
            }
            // ADD Vx, byte (7XNN) - Set Vx = Vx + NN (carry flag is NOT changed)
            (0x7, _, _, _) => {
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            // LD Vx, Vy (8XY0) - Set Vx = Vy
            (0x8, _, _, 0x0) => {
                self.v_reg[x] = self.v_reg[y];
            }
            // OR Vx, Vy (8XY1) - Set Vx = Vx OR Vy
            (0x8, _, _, 0x1) => {
                self.v_reg[x] |= self.v_reg[y];
            }
            // AND Vx, Vy (8XY2) - Set Vx = Vx AND Vy
            (0x8, _, _, 0x2) => {
                self.v_reg[x] &= self.v_reg[y];
            }
            // XOR Vx, Vy (8XY3) - Set Vx = Vx XOR Vy
            (0x8, _, _, 0x3) => {
                self.v_reg[x] ^= self.v_reg[y];
            }
            // ADD Vx, Vy (8XY4) - Set Vx = Vx + Vy, set VF = carry
            (0x8, _, _, 0x4) => {
                let (result, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                self.v_reg[x] = result;
                self.v_reg[0xF] = if carry { 1 } else { 0 };
            }
            // SUB Vx, Vy (8XY5) - Set Vx = Vx - Vy, set VF = NOT borrow
            (0x8, _, _, 0x5) => {
                let (result, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                self.v_reg[x] = result;
                // VF is 1 if there was NO borrow, 0 if there was a borrow
                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            }
            // SHR Vx {, Vy} (8XY6) - Set Vx = Vx SHR 1. VF = LSB of Vx before shift
            (0x8, _, _, 0x6) => {
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            // SUBN Vx, Vy (8XY7) - Set Vx = Vy - Vx, set VF = NOT borrow
            (0x8, _, _, 0x7) => {
                let (result, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                self.v_reg[x] = result;
                // VF is 1 if there was NO borrow, 0 if there was a borrow
                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            }
            // SHL Vx {, Vy} (8XYE) - Set Vx = Vx SHL 1. VF = MSB of Vx before shift
            (0x8, _, _, 0xE) => {
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }
            // SNE Vx, Vy (9XY0) - Skip next instruction if Vx != Vy
            (0x9, _, _, 0x0) => {
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // LD I, addr (ANNN) - Set I = NNN
            (0xA, _, _, _) => {
                self.i_reg = nnn;
            }
            // JP V0, addr (BNNN) - Jump to location NNN + V0
            (0xB, _, _, _) => {
                self.pc = (self.v_reg[0] as u16).wrapping_add(nnn);
            }
            // RND Vx, byte (CXNN) - Set Vx = random byte AND NN
            (0xC, _, _, _) => {
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            // DRW Vx, Vy, nibble (DXYN) - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            (0xD, _, _, _) => {
                // Get the coords (Vx, Vy) and wrap if necessary
                let x_coord = self.v_reg[x] as usize % SCREEN_WIDTH;
                let y_coord = self.v_reg[y] as usize % SCREEN_HEIGHT;
                let num_rows = n;

                // Assume no collision initially
                self.v_reg[0xF] = 0;

                // Iterate N rows
                for row in 0..num_rows {
                    // Stop drawing if bottom edge is reached
                    let py = y_coord + row;
                    if py >= SCREEN_HEIGHT {
                        break;
                    }

                    // Get sprite byte from RAM[I + row]
                    let sprite_byte = self.ram[(self.i_reg as usize + row) % RAM_SIZE]; // Ensure I doesn't wrap RAM

                    // Iterate 8 bits/pixels in the sprite byte
                    for bit in 0..8 {
                        // Stop drawing if right edge is reached
                        let px = x_coord + bit;
                        if px >= SCREEN_WIDTH {
                            break;
                        }

                        // Get the pixel bit (1 if sprite pixel is set, 0 otherwise)
                        // Check if the leftmost bit (bit 7) of sprite_byte is set
                        let sprite_pixel_is_on = (sprite_byte >> (7 - bit)) & 1;

                        // Only draw if the sprite pixel is set
                        if sprite_pixel_is_on != 0 {
                            // Calculate screen buffer index
                            let screen_idx = px + py * SCREEN_WIDTH;

                            // Check for collision: if screen pixel is already on
                            if self.screen[screen_idx] {
                                self.v_reg[0xF] = 1; // Set VF flag for collision
                            }
                            // XOR the pixel state: on -> off, off -> on
                            self.screen[screen_idx] ^= true;
                        }
                    }
                }
                // Optional: Indicate screen needs redraw
            }
            // SKP Vx (EX9E) - Skip next instruction if key with the value of Vx is pressed
            (0xE, _, 0x9, 0xE) => {
                let key_idx = self.v_reg[x] as usize;
                if key_idx < NUM_KEYS && self.keys[key_idx] {
                    // Check bounds and key state
                    self.pc += 2;
                }
            }
            // SKNP Vx (EXA1) - Skip next instruction if key with the value of Vx is NOT pressed
            (0xE, _, 0xA, 0x1) => {
                let key_idx = self.v_reg[x] as usize;
                // Check bounds and key state
                if key_idx >= NUM_KEYS || !self.keys[key_idx] {
                    self.pc += 2;
                }
            }
            // LD Vx, DT (FX07) - Set Vx = delay timer value
            (0xF, _, 0x0, 0x7) => {
                self.v_reg[x] = self.dt;
            }
            // LD Vx, K (FX0A) - Wait for a key press, store the value of the key in Vx
            (0xF, _, 0x0, 0xA) => {
                let mut key_pressed = false;
                for (i, key_state) in self.keys.iter().enumerate() {
                    if *key_state {
                        self.v_reg[x] = i as u8;
                        key_pressed = true;
                        break; // Found first pressed key
                    }
                }
                // If no key was pressed, repeat this instruction
                if !key_pressed {
                    // Decrement PC by 2 because fetch() already incremented it
                    self.pc -= 2;
                }
            }
            // LD DT, Vx (FX15) - Set delay timer = Vx
            (0xF, _, 0x1, 0x5) => {
                self.dt = self.v_reg[x];
            }
            // LD ST, Vx (FX18) - Set sound timer = Vx
            (0xF, _, 0x1, 0x8) => {
                self.st = self.v_reg[x];
            }
            // ADD I, Vx (FX1E) - Set I = I + Vx
            (0xF, _, 0x1, 0xE) => {
                // CHIP-8 specs are ambiguous about I overflowing 0xFFF.
                // Some emulators wrap I, some don't set VF, some set VF on overflow.
                // Wrapping add is a common approach.
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
                // Optionally handle the undocumented overflow flag behavior if needed for specific games
                // if self.i_reg < (self.v_reg[x] as u16) { /* handle overflow if required */ }
            }
            // LD F, Vx (FX29) - Set I = location of sprite for digit Vx
            (0xF, _, 0x2, 0x9) => {
                // Font characters are 5 bytes high. Assumes FONTSET starts at 0.
                let digit = self.v_reg[x] as u16;
                // Make sure digit is 0-F
                self.i_reg = (digit & 0xF) * 5;
            }
            // LD B, Vx (FX33) - Store BCD representation of Vx in memory locations I, I+1, and I+2
            (0xF, _, 0x3, 0x3) => {
                let vx = self.v_reg[x]; // Get the value from Vx (u8)

                let hundreds = vx / 100; // Integer division gives the hundreds digit
                let tens = (vx / 10) % 10; // Get tens digit
                let ones = vx % 10; // Get ones digit

                // Ensure I is within bounds
                let i = self.i_reg as usize;
                if i + 2 < RAM_SIZE {
                    self.ram[i] = hundreds;
                    self.ram[i + 1] = tens;
                    self.ram[i + 2] = ones;
                } else {
                    // Handle error: attempt to write BCD out of bounds
                    // e.g., panic!("BCD write out of bounds"); or log an error
                }
            }
            // LD [I], Vx (FX55) - Store registers V0 through Vx in memory starting at location I
            (0xF, _, 0x5, 0x5) => {
                let i = self.i_reg as usize;
                // Ensure we don't write past RAM end
                if i + x < RAM_SIZE {
                    for idx in 0..=x {
                        self.ram[i + idx] = self.v_reg[idx];
                    }
                } else {
                    // Handle error: write out of bounds
                }
            }
            // LD Vx, [I] (FX65) - Read registers V0 through Vx from memory starting at location I
            (0xF, _, 0x6, 0x5) => {
                let i = self.i_reg as usize;
                // Ensure we don't read past RAM end
                if i + x < RAM_SIZE {
                    for idx in 0..=x {
                        self.v_reg[idx] = self.ram[i + idx];
                    }
                }
            }
            _ => unimplemented!("We didn't implement all the instructions yet"),
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
        if idx >= NUM_KEYS {
            eprintln!(
                "Warning: Key index {} out of bounds (0-{})",
                idx,
                NUM_KEYS - 1
            );
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
