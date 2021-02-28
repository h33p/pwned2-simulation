mod tape;
use tape::create_tape;

struct InputTable {
    registers: [u8; 4],
    c6: bool,
    stack_ptr: *mut u8,
    tape_pos: *mut u8,
    identity: *mut u8,
    target_key_index: usize,
    target_key: [u8; 31],
}

impl InputTable {
    pub unsafe fn new(input: *mut u8) -> Self {
        Self {
            registers: [0; 4],
            c6: false,
            stack_ptr: input,
            tape_pos: input.sub(1),
            identity: input.sub(1),
            target_key_index: 30,
            target_key: [
                0x79, 0x73, 0x4E, 0x2B, 0x23, 0x0B4, 0x5B, 0x33, 0x35, 0x7E, 0x7A, 0x13, 0x69,
                0x74, 0x4E, 0x10, 0x52, 0x76, 0x0FA, 0x0EC, 0x69, 0x0, 0x2C, 0x35, 0x75, 0x64,
                0x18, 0x57, 0x33, 0x78, 0xED,
            ],
        }
    }

    unsafe fn pop_reg_op(&mut self) {
        let p1 = self.tape_pos;
        let p2 = self.identity;
        if p1 != p2 {
            let c = *self.stack_ptr.add(1) as usize;
            if c > 0 && c <= 4 {
                self.registers[c - 1] = *self.tape_pos;
            }
        }
        self.tape_pos = self.tape_pos.sub(1);
        self.stack_ptr = self.stack_ptr.add(2);
    }

    // This gets the xor key out to the registers
    unsafe fn read_secret_op(&mut self) {
        let c = *self.stack_ptr.add(1) as usize;
        if c > 0 && c <= 4 {
            self.registers[c - 1] = self.target_key[self.target_key_index];
        }
        self.target_key_index -= 1;
        self.stack_ptr = self.stack_ptr.add(2);
    }

    unsafe fn push_mem_op(&mut self) {
        let c = *self.stack_ptr.add(1);
        self.tape_pos = self.tape_pos.add(1);
        // Override tape...
        *self.tape_pos = c;
        self.stack_ptr = self.stack_ptr.add(2);
    }

    unsafe fn push_reg_op(&mut self) {
        let c = *self.stack_ptr.add(1) as usize;
        let v = if c > 0 && c <= 4 {
            self.registers[c - 1]
        } else {
            0
        };
        self.tape_pos = self.tape_pos.add(1);
        *self.tape_pos = v;
        self.stack_ptr = self.stack_ptr.add(2);
    }

    // case 4 we never should hit...

    unsafe fn cmp_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let c2 = *self.stack_ptr.add(2) as usize;

        let v1 = if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1]
        } else {
            0
        };

        let v2 = if c2 > 0 && c2 <= 4 {
            self.registers[c2 - 1]
        } else {
            0
        };

        self.c6 = v1 == v2;
        self.stack_ptr = self.stack_ptr.add(3);
    }

    unsafe fn mov_op(&mut self) {
        let c2 = *self.stack_ptr.add(2) as usize;

        let v2 = if c2 > 0 && c2 <= 4 {
            self.registers[c2 - 1]
        } else {
            0
        };

        let c1 = *self.stack_ptr.add(1) as usize;
        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = v2;
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    // Skips variable amount of steps...
    unsafe fn jump_if_op(&mut self, variable: bool) {
        if variable {
            let v = *self.stack_ptr.add(1) as usize;
            self.stack_ptr = self.stack_ptr.add(v);
        } else {
            self.stack_ptr = self.stack_ptr.add(2);
        }
    }

    unsafe fn je_op(&mut self) {
        self.jump_if_op(self.c6);
    }

    unsafe fn jne_op(&mut self) {
        self.jump_if_op(!self.c6);
    }

    // subs ptr by the value
    // we should probs avoid this too... OR USE FOR OURSELVES
    unsafe fn jmp_back_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        self.stack_ptr = self.stack_ptr.sub(c1);
    }

    // xors!
    unsafe fn xor_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let xor_key = *self.stack_ptr.add(2);

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] ^= xor_key;
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    // rotate bits
    unsafe fn ror_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let rotate_by = *self.stack_ptr.add(2) as u32;

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = self.registers[c1 - 1].rotate_right(rotate_by);
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    unsafe fn rol_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let rotate_by = *self.stack_ptr.add(2) as u32;

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = self.registers[c1 - 1].rotate_left(rotate_by);
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    // arithmetic!
    unsafe fn add_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let add_val = *self.stack_ptr.add(2);

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = self.registers[c1 - 1].wrapping_add(add_val);
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    unsafe fn sub_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let sub_val = *self.stack_ptr.add(2);

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = self.registers[c1 - 1].wrapping_sub(sub_val);
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    unsafe fn mul_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;
        let mul_val = *self.stack_ptr.add(2);

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] = self.registers[c1 - 1].wrapping_mul(mul_val);
        }

        self.stack_ptr = self.stack_ptr.add(3);
    }

    unsafe fn inc_op(&mut self) {
        let c1 = *self.stack_ptr.add(1) as usize;

        if c1 > 0 && c1 <= 4 {
            self.registers[c1 - 1] += 1;
        }

        self.stack_ptr = self.stack_ptr.add(2);
    }
}

unsafe fn unsafe_main() {
    let alphabet = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_ ";

    let mut key: [u8; 31] = *b"pwned{aaaaaaaaaaaaaaaaaaaaaaaa}";

    loop {
        let mut tape = create_tape(key);

        let mut ctx = InputTable::new(tape.as_mut_ptr());

        let success = loop {
            match *ctx.stack_ptr {
                0 => ctx.pop_reg_op(),
                1 => ctx.read_secret_op(),
                2 => ctx.push_mem_op(),
                3 => ctx.push_reg_op(),
                4 => break false,
                5 => ctx.cmp_op(),
                6 => ctx.mov_op(),
                7 => ctx.je_op(),
                8 => ctx.jne_op(),
                9 => ctx.jmp_back_op(),
                10 => ctx.xor_op(),
                11 => ctx.ror_op(),
                12 => ctx.rol_op(),
                13 => ctx.add_op(),
                14 => ctx.sub_op(),
                15 => ctx.mul_op(),
                16 => ctx.inc_op(),
                _ => break true,
            }
        };

        if success {
            println!("Success!!!");

            for &k in &key {
                print!("{}", k as char);
            }

            println!();

            break;
        } else {
            println!("KEY FAILED {}", std::str::from_utf8(&key).unwrap());
            for i in 0..31 {
                if tape[i] != ctx.target_key[i] {
                    let pos = alphabet
                        .iter()
                        .copied()
                        .enumerate()
                        .find(|(_, e)| *e == key[i])
                        .unwrap()
                        .0;
                    key[i] = alphabet[pos + 1];
                }
            }
        }
    }
}

fn main() {
    unsafe {
        unsafe_main();
    }
}
