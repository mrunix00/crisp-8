use rand::{RngExt, rngs::ThreadRng};

const FONT_ADDR: u16 = 0x050;
const PROGRAM_ADDR: u16 = 0x200;
const MASK_12BIT: u16 = 0x0fff;

const FONT: [u8; 80] = [
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

pub struct Chip8VM {
    pub display: [u8; 64 * 32],
    memory: [u8; 4096],
    registers: [u8; 16],
    stack: [u16; 16],
    stack_pointer: u8,
    index_register: u16,
    program_counter: u16,
    dt: u8,
    st: u8,
    keypad: [bool; 16],
    rng: ThreadRng,
}

impl Chip8VM {
    pub fn new(program: &[u8]) -> Chip8VM {
        let mut memory = [0u8; 4096];
        let len = program.len().min(4096 - PROGRAM_ADDR as usize);
        memory[FONT_ADDR as usize..FONT_ADDR as usize + FONT.len()].copy_from_slice(&FONT);
        memory[PROGRAM_ADDR as usize..PROGRAM_ADDR as usize + len].copy_from_slice(&program[..len]);
        Chip8VM {
            memory,
            display: [0; 64 * 32],
            registers: [0; 16],
            stack: [0; 16],
            stack_pointer: 0,
            index_register: 0,
            program_counter: PROGRAM_ADDR,
            dt: 0,
            st: 0,
            keypad: [false; 16],
            rng: rand::rng(),
        }
    }

    pub fn key_down(&mut self, key: u8) {
        self.keypad[key as usize] = true;
    }

    pub fn key_up(&mut self, key: u8) {
        self.keypad[key as usize] = false;
    }

    fn get_register(&self, word: u8) -> u8 {
        self.registers[(word & 0x0f) as usize]
    }

    fn set_register(&mut self, word: u8, value: u8) {
        self.registers[(word & 0x0f) as usize] = value;
    }

    fn set_flag(&mut self, value: u8) {
        self.set_register(15, value);
    }

    fn op_cls(&mut self) {
        self.display.fill(0);
    }

    fn op_ret(&mut self) {
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.program_counter = self.stack[self.stack_pointer as usize] & MASK_12BIT;
    }

    fn op_jmp(&mut self, word1: u8, word2: u8) {
        self.program_counter = ((word1 & 0x0f) as u16) << 8 | word2 as u16;
    }

    fn op_call(&mut self, word1: u8, word2: u8) {
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer = (self.stack_pointer + 1) & 0x0f;
        self.program_counter = ((word1 & 0x0f) as u16) << 8 | word2 as u16;
    }

    fn op_se_kv(&mut self, word1: u8, word2: u8) {
        if self.get_register(word1) == word2 {
            self.program_counter += 2;
        }
    }

    fn op_sne_kv(&mut self, word1: u8, word2: u8) {
        if self.get_register(word1) != word2 {
            self.program_counter += 2;
        }
    }

    fn op_se_vv(&mut self, word1: u8, word2: u8) {
        if self.get_register(word1) == self.get_register(word2 >> 4) {
            self.program_counter += 2;
        }
    }

    fn op_ld_kv(&mut self, word1: u8, word2: u8) {
        self.set_register(word1, word2);
    }

    fn op_add_kv(&mut self, word1: u8, word2: u8) {
        self.set_register(word1, self.get_register(word1).wrapping_add(word2));
    }

    fn op_ld_vv(&mut self, word1: u8, word2: u8) {
        self.set_register(word1, self.get_register(word2 >> 4));
    }

    fn op_or(&mut self, word1: u8, word2: u8) {
        self.set_register(
            word1,
            self.get_register(word1) | self.get_register(word2 >> 4),
        );
    }

    fn op_and(&mut self, word1: u8, word2: u8) {
        self.set_register(
            word1,
            self.get_register(word1) & self.get_register(word2 >> 4),
        );
    }

    fn op_xor(&mut self, word1: u8, word2: u8) {
        self.set_register(
            word1,
            self.get_register(word1) ^ self.get_register(word2 >> 4),
        );
    }

    fn op_add_vv(&mut self, word1: u8, word2: u8) {
        let sum = self.get_register(word1) as u16 + self.get_register(word2 >> 4) as u16;
        self.set_register(word1, sum as u8);
        self.set_flag((sum > 0xff) as u8);
    }

    fn op_sub_vv(&mut self, word1: u8, word2: u8) {
        let x = self.get_register(word1);
        let y = self.get_register(word2 >> 4);
        self.set_register(word1, x.wrapping_sub(y));
        self.set_flag((x >= y) as u8);
    }

    fn op_shr(&mut self, word1: u8) {
        let x = self.get_register(word1);
        self.set_register(word1, x >> 1);
        self.set_flag(x & 1);
    }

    fn op_shl(&mut self, word1: u8) {
        let x = self.get_register(word1);
        self.set_register(word1, x << 1);
        self.set_flag(x >> 7);
    }

    fn op_subn_vv(&mut self, word1: u8, word2: u8) {
        let x = self.get_register(word1);
        let y = self.get_register(word2 >> 4);
        self.set_register(word1, y.wrapping_sub(x));
        self.set_flag((y >= x) as u8);
    }

    fn op_sne_vv(&mut self, word1: u8, word2: u8) {
        if self.get_register(word1) != self.get_register(word2 >> 4) {
            self.program_counter += 2;
        }
    }

    fn op_ld_in(&mut self, word1: u8, word2: u8) {
        self.index_register = ((word1 & 0x0f) as u16) << 8 | word2 as u16;
    }

    fn op_jp_va(&mut self, word1: u8, word2: u8) {
        let nnn = ((word1 & 0x0f) as u16) << 8 | word2 as u16;
        self.program_counter = (nnn + self.get_register(0) as u16) & MASK_12BIT;
    }

    fn op_rnd_vb(&mut self, word1: u8, word2: u8) {
        let rnd = self.rng.random::<u8>();
        self.set_register(word1, word2 & rnd);
    }

    fn op_drw_vvn(&mut self, word1: u8, word2: u8) {
        let x = self.get_register(word1) as usize % 64;
        let y = self.get_register(word2 >> 4) as usize % 32;
        let rows = (word2 & 0x0f) as usize;
        self.set_flag(0);
        for row in 0..rows {
            let sprite = self.memory[(self.index_register as usize + row) % self.memory.len()];
            for col in 0..8 {
                if sprite & (0x80 >> col) == 0 {
                    continue;
                }
                let idx = (y + row) % 32 * 64 + (x + col) % 64;
                self.display[idx] ^= 1;
                if self.display[idx] == 0 {
                    self.set_flag(1);
                }
            }
        }
    }

    fn op_skp_vx(&mut self, word1: u8) {
        if self.keypad[self.get_register(word1) as usize] {
            self.program_counter += 2;
        }
    }

    fn op_sknp_vx(&mut self, word1: u8) {
        if !self.keypad[self.get_register(word1) as usize] {
            self.program_counter += 2;
        }
    }

    fn op_ld_vx_dt(&mut self, word1: u8) {
        self.set_register(word1, self.dt);
    }

    fn op_ld_key(&mut self, word1: u8) {
        if let Some(key) = self.keypad.iter().position(|&k| k) {
            self.set_register(word1, key as u8);
        } else {
            self.program_counter -= 2;
        }
    }

    fn op_ld_dt_vx(&mut self, word1: u8) {
        self.dt = self.get_register(word1);
    }

    fn op_ld_st_vx(&mut self, word1: u8) {
        self.st = self.get_register(word1);
    }

    fn op_add_i_vx(&mut self, word1: u8) {
        self.index_register = (self.index_register + self.get_register(word1) as u16) & MASK_12BIT;
    }

    fn op_ld_f_vx(&mut self, word1: u8) {
        self.index_register = FONT_ADDR + 5 * self.get_register(word1) as u16;
    }

    fn op_ld_b_vx(&mut self, word1: u8) {
        let value = self.get_register(word1);
        let bcd = [value / 100, value / 10 % 10, value % 10];
        for (offset, digit) in bcd.iter().enumerate() {
            let addr = (self.index_register as usize + offset) % self.memory.len();
            self.memory[addr] = *digit;
        }
    }

    fn op_ld_i_vx(&mut self, word1: u8) {
        let x = (word1 & 0x0f) as usize;
        for offset in 0..=x {
            let addr = (self.index_register as usize + offset) % self.memory.len();
            self.memory[addr] = self.registers[offset];
        }
    }

    fn op_ld_vx_i(&mut self, word1: u8) {
        let x = (word1 & 0x0f) as usize;
        for offset in 0..=x {
            let addr = (self.index_register as usize + offset) % self.memory.len();
            self.registers[offset] = self.memory[addr];
        }
    }

    pub fn exec(&mut self, word1: u8, word2: u8) {
        self.program_counter = (self.program_counter + 2) & MASK_12BIT;
        match word1 & 0xf0 {
            0x00 => match word2 {
                0xe0 => self.op_cls(),
                0xee => self.op_ret(),
                _ => {}
            },
            0x10 => self.op_jmp(word1, word2),
            0x20 => self.op_call(word1, word2),
            0x30 => self.op_se_kv(word1, word2),
            0x40 => self.op_sne_kv(word1, word2),
            0x50 => self.op_se_vv(word1, word2),
            0x60 => self.op_ld_kv(word1, word2),
            0x70 => self.op_add_kv(word1, word2),
            0x80 => match word2 & 0x0f {
                0x00 => self.op_ld_vv(word1, word2),
                0x01 => self.op_or(word1, word2),
                0x02 => self.op_and(word1, word2),
                0x03 => self.op_xor(word1, word2),
                0x04 => self.op_add_vv(word1, word2),
                0x05 => self.op_sub_vv(word1, word2),
                0x06 => self.op_shr(word1),
                0x07 => self.op_subn_vv(word1, word2),
                0x0e => self.op_shl(word1),
                _ => {}
            },
            0x90 => self.op_sne_vv(word1, word2),
            0xa0 => self.op_ld_in(word1, word2),
            0xb0 => self.op_jp_va(word1, word2),
            0xc0 => self.op_rnd_vb(word1, word2),
            0xd0 => self.op_drw_vvn(word1, word2),
            0xe0 => match word2 {
                0x9e => self.op_skp_vx(word1),
                0xa1 => self.op_sknp_vx(word1),
                _ => {}
            },
            0xf0 => match word2 {
                0x07 => self.op_ld_vx_dt(word1),
                0x0a => self.op_ld_key(word1),
                0x15 => self.op_ld_dt_vx(word1),
                0x18 => self.op_ld_st_vx(word1),
                0x1e => self.op_add_i_vx(word1),
                0x29 => self.op_ld_f_vx(word1),
                0x33 => self.op_ld_b_vx(word1),
                0x55 => self.op_ld_i_vx(word1),
                0x65 => self.op_ld_vx_i(word1),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        let pc = (self.program_counter & MASK_12BIT) as usize;
        self.exec(self.memory[pc], self.memory[(pc + 1) % self.memory.len()]);
    }

    pub fn timer_tick(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }
}
