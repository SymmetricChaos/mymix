use std::{cmp::Ordering, fmt::Display};

pub trait Word {
    fn as_word(&self) -> String;
    fn op_code(&self) -> u8;
    fn modifier(&self) -> u8;
    fn index(&self) -> u8;
    fn address_bytes(&self) -> [u8; 5];
    fn address(&self) -> usize;
    fn fields(&self) -> (usize, u8, u8, u8) {
        (
            self.address(),
            self.index(),
            self.modifier(),
            self.op_code(),
        )
    }
}

impl Word for u64 {
    fn as_word(&self) -> String {
        format!("{:?}", self.to_be_bytes())
    }

    fn op_code(&self) -> u8 {
        self.to_be_bytes()[7]
    }

    fn modifier(&self) -> u8 {
        self.to_be_bytes()[6]
    }

    fn index(&self) -> u8 {
        self.to_be_bytes()[5]
    }

    fn address_bytes(&self) -> [u8; 5] {
        let mut addr = [0, 0, 0, 0, 0];
        addr.clone_from_slice(&self.to_be_bytes()[0..5]);
        addr
    }

    fn address(&self) -> usize {
        (self / 16777216) as usize
    }
}

pub struct MyMix {
    a: u64,
    x: u64,
    i: [u64; 6],
    j: u64,
    cmp: Ordering,
    mem: Box<[u64; 100]>,
    instr_ptr: usize,
}

impl Display for MyMix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let registers = format!(
            "Registers\na: {}\nx: {}\ni1 {}\ni2: {}\ni3: {}\ni4: {}\ni5: {}\ni6: {}\nj: {}\ncmp: {:?}",
            self.a.as_word(),
            self.x.as_word(),
            self.i[0].as_word(),
            self.i[1].as_word(),
            self.i[2].as_word(),
            self.i[3].as_word(),
            self.i[4].as_word(),
            self.i[5].as_word(),
            self.j.as_word(),
            self.cmp
        );
        let mut memory = String::from("Memory\n");
        for (n, line) in self.mem.iter().enumerate() {
            if line != &0 {
                memory.push_str(&format!("{}: {:?}\n", n, line.as_word()));
            }
        }
        let state = format!("{}\n\n{}", registers, memory);
        write!(f, "{}", state)
    }
}

impl MyMix {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            i: [0, 0, 0, 0, 0, 0],
            j: 0,
            mem: Box::new([0; 100]),
            cmp: Ordering::Equal,
            instr_ptr: 1,
        }
    }

    pub fn print_mem(&self) {
        let mut memory = String::from("Memory\n");
        for (n, line) in self.mem.iter().enumerate() {
            if line != &0 {
                memory.push_str(&format!("{}: {:?}", n, line.as_word()));
                if n == self.instr_ptr {
                    memory.push_str("  <=");
                }
                memory.push('\n');
            }
        }
        println!("{}", memory)
    }

    //let (address, index, modification, op_code) = instruction.fields();

    pub fn load(&mut self, instruction: u64) {
        let (address, index, _, op_code) = instruction.fields();

        let addr = if index != 0 {
            address.wrapping_add(self.i[(index - 1) as usize] as usize)
        } else {
            address
        };

        match op_code {
            8 => self.a = self.mem[addr],
            9 => self.i[0] = self.mem[addr],
            10 => self.i[1] = self.mem[addr],
            11 => self.i[2] = self.mem[addr],
            12 => self.i[3] = self.mem[addr],
            13 => self.i[4] = self.mem[addr],
            14 => self.i[5] = self.mem[addr],
            15 => self.x = self.mem[addr],
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }

    pub fn store(&mut self, instruction: u64) {
        let (address, index, _, op_code) = instruction.fields();

        let addr = if index != 0 {
            address.wrapping_add(self.i[(index - 1) as usize] as usize)
        } else {
            address
        };

        match op_code {
            24 => self.mem[addr] = self.a,
            25 => self.mem[addr] = self.i[0],
            26 => self.mem[addr] = self.i[1],
            27 => self.mem[addr] = self.i[2],
            28 => self.mem[addr] = self.i[3],
            29 => self.mem[addr] = self.i[4],
            30 => self.mem[addr] = self.i[5],
            31 => self.mem[addr] = self.x,
            32 => self.mem[addr] = self.j,
            33 => self.mem[addr] = 0,
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }

    pub fn arith(&mut self, instruction: u64) {
        let (address, index, _, op_code) = instruction.fields();

        let addr = address + self.i[index as usize] as usize;

        match op_code {
            1 => self.a = self.a.wrapping_add(self.mem[addr]),
            2 => self.a = self.a.wrapping_sub(self.mem[addr]),
            3 => self.a = self.a.wrapping_mul(self.mem[addr]),
            4 => self.a = self.a.wrapping_div(self.mem[addr]),
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }

    pub fn inc_dec_ent(&mut self, instruction: u64) {
        let (address, index, modification, op_code) = instruction.fields();

        let addr = if index != 0 {
            address.wrapping_add(self.i[(index - 1) as usize] as usize) as u64
        } else {
            address as u64
        };

        // Increment
        if modification == 0 {
            match op_code {
                48 => self.a = self.a.wrapping_add(addr),
                49 => self.i[0] = self.i[0].wrapping_add(addr),
                50 => self.i[1] = self.i[1].wrapping_add(addr),
                51 => self.i[2] = self.i[2].wrapping_add(addr),
                52 => self.i[3] = self.i[3].wrapping_add(addr),
                53 => self.i[4] = self.i[4].wrapping_add(addr),
                54 => self.i[5] = self.i[5].wrapping_add(addr),
                55 => self.x = self.x.wrapping_add(addr),
                _ => panic!("unknown instruction: {}", instruction.as_word()),
            }
        // Decrement
        } else if modification == 1 {
            match op_code {
                48 => self.a = self.a.wrapping_sub(addr),
                49 => self.i[0] = self.i[0].wrapping_sub(addr),
                50 => self.i[1] = self.i[1].wrapping_sub(addr),
                51 => self.i[2] = self.i[2].wrapping_sub(addr),
                52 => self.i[3] = self.i[3].wrapping_sub(addr),
                53 => self.i[4] = self.i[4].wrapping_sub(addr),
                54 => self.i[5] = self.i[5].wrapping_sub(addr),
                55 => self.x = self.x.wrapping_sub(addr),
                _ => panic!("unknown instruction: {}", instruction.as_word()),
            }
        // Enter
        } else if modification == 2 {
            match op_code {
                48 => self.a = addr,
                49 => self.i[0] = addr,
                50 => self.i[1] = addr,
                51 => self.i[2] = addr,
                52 => self.i[3] = addr,
                53 => self.i[4] = addr,
                54 => self.i[5] = addr,
                55 => self.x = addr,
                _ => panic!("unknown instruction: {}", instruction.as_word()),
            }
        } else {
            panic!("unknown instruction: {}", instruction.as_word())
        }
    }

    pub fn cmp(&mut self, instruction: u64) {
        let (address, index, _, op_code) = instruction.fields();

        let addr = if index != 0 {
            address.wrapping_add(self.i[(index - 1) as usize] as usize)
        } else {
            address
        };

        match op_code {
            56 => self.cmp = self.a.cmp(&self.mem[addr]),
            57 => self.cmp = self.i[0].cmp(&self.mem[addr]),
            58 => self.cmp = self.i[1].cmp(&self.mem[addr]),
            59 => self.cmp = self.i[2].cmp(&self.mem[addr]),
            60 => self.cmp = self.i[3].cmp(&self.mem[addr]),
            61 => self.cmp = self.i[4].cmp(&self.mem[addr]),
            62 => self.cmp = self.i[5].cmp(&self.mem[addr]),
            63 => self.cmp = self.x.cmp(&self.mem[addr]),
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }

    fn save_and_jump(&mut self, addr: usize) {
        self.j = (self.instr_ptr + 1) as u64;
        self.instr_ptr = addr as usize
    }

    pub fn jump(&mut self, instruction: u64) {
        let (address, index, modifier, op_code) = instruction.fields();

        let addr = if index != 0 {
            address.wrapping_add(self.i[(index - 1) as usize] as usize)
        } else {
            address
        };
        match op_code {
            39 => match modifier {
                0 => self.save_and_jump(addr),
                1 => self.instr_ptr = addr as usize,
                4 => {
                    if self.cmp == Ordering::Less {
                        self.save_and_jump(addr)
                    }
                }
                5 => {
                    if self.cmp == Ordering::Equal {
                        self.save_and_jump(addr)
                    }
                }
                6 => {
                    if self.cmp == Ordering::Greater {
                        self.save_and_jump(addr)
                    }
                }
                7 => {
                    if self.cmp != Ordering::Less {
                        self.save_and_jump(addr)
                    }
                }
                8 => {
                    if self.cmp != Ordering::Equal {
                        self.save_and_jump(addr)
                    }
                }
                9 => {
                    if self.cmp != Ordering::Greater {
                        self.save_and_jump(addr)
                    }
                }
                _ => panic!("unknown instruction: {}", instruction.as_word()),
            },
            40 => todo!(),
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }

    pub fn step(&mut self) {
        if self.instr_ptr == 0 {
            return;
        }
        self.read();
        if self.instr_ptr == 0 {
            return;
        }
        self.instr_ptr += 1;
    }

    pub fn run(&mut self) {
        loop {
            if self.instr_ptr == 0 {
                break;
            }
            self.read();
            if self.instr_ptr == 0 {
                break;
            }
            self.instr_ptr += 1;
        }
    }

    pub fn read(&mut self) {
        let instruction = self.mem[self.instr_ptr];
        match instruction.op_code() {
            1..=4 => self.arith(instruction),
            8..=15 => self.load(instruction),
            24..=33 => self.store(instruction),
            39..=47 => self.jump(instruction),
            48..=55 => self.inc_dec_ent(instruction),
            56..=63 => self.cmp(instruction),
            _ => panic!("unknown instruction: {}", instruction.as_word()),
        }
    }
}

fn main() {
    let mut computer = MyMix::new();
    computer.mem[1] = u64::from_be_bytes([0, 0, 0, 0, 10, 0, 0, 8]); // LOAD M10 into rA
    computer.mem[2] = u64::from_be_bytes([0, 0, 0, 0, 10, 1, 0, 1]); // ADD rA to M10 + i1
    computer.mem[3] = u64::from_be_bytes([0, 0, 0, 0, 11, 1, 0, 24]); // STORE rA at M11 + i1
    computer.mem[4] = u64::from_be_bytes([0, 0, 0, 0, 1, 0, 0, 49]); // INC i1
    computer.mem[5] = u64::from_be_bytes([0, 0, 0, 0, 50, 0, 0, 57]); // CMP i1 to M50
    computer.mem[6] = u64::from_be_bytes([0, 0, 0, 0, 1, 0, 8, 39]); // JUMP to M1 if not equal
    computer.mem[7] = u64::from_be_bytes([0, 0, 0, 0, 0, 0, 1, 39]); // JUMP to M0
    computer.mem[50] = u64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 5]); // constant 5
    computer.mem[10] = u64::from_be_bytes([1, 2, 3, 5, 8, 13, 21, 37]); // some bytes
    println!("{}", computer);
    computer.run();
    println!("{}", computer);
    // for _ in 0..10 {
    //     computer.step();
    //     computer.print_mem();
    //     computer.step();
    //     computer.print_mem();
    //     computer.step();
    //     computer.print_mem();
    //     computer.step();
    //     computer.print_mem();
    //     computer.step();
    //     computer.print_mem();
    //     computer.step();
    //     computer.print_mem();
    // }
}
