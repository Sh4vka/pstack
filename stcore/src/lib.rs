#[derive(Debug, PartialEq)]
pub enum Command {
    PUSH,
    POP,
    READ,
    SWAP,
    ROT,
    DUP,
    LDC,
    LOOP,
    ADD,
    INC,
    MUL,
    NOP,
    HALT
}

impl Command {
    pub fn from(byte: i8) -> Self {
        match byte {
            0 => Command::PUSH,
            1 => Command::POP,
            2 => Command::SWAP,
            3 => Command::ROT,
            4 => Command::DUP,
            5 => Command::LDC,
            6 => Command::LOOP,
            7 => Command::ADD,
            8 => Command::INC,
            9 => Command::MUL,
            10 => Command::READ,
            -0x7f => Command::NOP,
            11 => Command::HALT,
            _ => panic!("Error parse command")
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    FETCH,
    DECODE,
    EXECUTE,
    WRITE,
}

pub const STACKSIZE: usize = 124;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Flags { pub z: bool, pub n: bool, pub c: bool, pub v: bool }

fn alu_add(a: i8, b: i8) -> (i8, Flags) {
    let (res, v) = a.overflowing_add(b);
    let (_, c)   = (a as u8).overflowing_add(b as u8);
    (res, Flags { z: res == 0, n: res < 0, c, v })
}

fn alu_mul(a: i8, b: i8) -> (i8, Flags) {
    let (res, v) = a.overflowing_mul(b);
    (res, Flags { z: res == 0, n: res < 0, c: v, v })
}

pub struct Core {
    pub stack: [i8; STACKSIZE],
    pub sp: isize,
    pub cnt: i8,
    pub op1: i8,
    pub op2: i8,
    pub op3: i8,
    pub res: i8,
    pub lit: i8,
    pub pc: usize,
    pub addr: usize,
    pub cmd: Command,
    pub state: State,
    pub halt: bool,
    pub flags: Flags,
    pub alu_flags: Flags,
}

impl Core {
    pub fn new() -> Self {
        Core {
            stack: [-0x7f_i8; STACKSIZE],
            sp: -1,
            cnt: 0,
            op1: 0,
            op2: 0,
            op3: 0,
            res: 0,
            lit: 0,
            pc: 0,
            addr: 0,
            cmd: Command::NOP,
            state: State::FETCH,
            halt: false,
            flags: Flags::default(),
            alu_flags: Flags::default(),
        }
    }

    pub fn reset(&mut self) {
        *self = Core::new();
    }

    pub fn step(&mut self, prog: &Vec<i8>, mem: &Vec<i8>) {
        if self.halt { return; }
        match self.state {
            State::FETCH => {
                self.fetch(prog);
                self.state = State::DECODE;
            },
            State::DECODE => {
                self.decode(prog);
                self.state = State::EXECUTE;
            },
            State::EXECUTE => {
                self.execute(mem);
                self.state = State::WRITE;
            },
            State::WRITE => {
                self.write();
                self.state = State::FETCH;
            }
        }
        self.deblog();
    }

    pub fn next(&mut self, prog: &Vec<i8>, mem: &Vec<i8>) {
        while self.state != State::WRITE {
            self.step(prog, mem);
        }
        self.step(prog, mem);
    }

    fn fetch(&mut self, prog: &Vec<i8>) {
        self.cmd = Command::from(prog[self.pc]);
    }

    fn decode(&mut self, prog: &Vec<i8>) {
        match self.cmd {
            Command::PUSH => self.lit = prog[self.pc + 1],
            Command::DUP => self.op1 = self.stack[self.sp as usize],
            Command::INC => self.op1 = self.stack[self.sp as usize],
            Command::READ => self.op1 = self.stack[self.sp as usize],
            Command::LDC => self.op1 = self.stack[self.sp as usize],
            Command::ADD => {
                self.op1 = self.stack[self.sp as usize];
                self.op2 = self.stack[(self.sp - 1) as usize];
            }
            Command::MUL => {
                self.op1 = self.stack[self.sp as usize];
                self.op2 = self.stack[(self.sp - 1) as usize];
            }
            Command::SWAP => {
                self.op1 = self.stack[self.sp as usize];
                self.op2 = self.stack[(self.sp - 1) as usize];
            },
            Command::ROT => {
                self.op1 = self.stack[self.sp as usize];
                self.op2 = self.stack[(self.sp - 1) as usize];
                self.op3 = self.stack[(self.sp - 2) as usize];
            },
            Command::LOOP => {
                self.op1 = self.cnt;
                self.addr = prog[self.pc + 1] as usize;
            }
            _ => {}
        }
    }

    fn execute(&mut self, mem: &Vec<i8>) {
        match self.cmd {
            Command::DUP => self.res = self.op1,
            Command::ADD => { let (r, f) = alu_add(self.op2, self.op1); self.res = r; self.alu_flags = f; }
            Command::MUL => { let (r, f) = alu_mul(self.op2, self.op1); self.res = r; self.alu_flags = f; }
            Command::INC => { let (r, f) = alu_add(self.op1, 1);        self.res = r; self.alu_flags = f; }
            Command::READ => self.res = mem[self.op1 as usize],
            Command::LOOP => self.res = self.op1 - 1,
            _ => {}
        }
    }

    fn write(&mut self) {
        match self.cmd {
            Command::PUSH => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.lit;
                self.pc += 2;
            },
            Command::POP => {
                self.sp -= 1;
                self.pc += 1
            },
            Command::DUP => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.res;
                self.pc += 1;
            },
            Command::SWAP => {
                self.stack[self.sp as usize] = self.op2;
                self.stack[(self.sp - 1) as usize] = self.op1;
                self.pc += 1;
            },
            Command::ROT => {
                self.stack[(self.sp - 2) as usize] = self.op2;
                self.stack[(self.sp - 1) as usize] = self.op1;
                self.stack[self.sp as usize] = self.op3;
                self.pc += 1;
            },
            Command::ADD => {
                self.sp -= 1;
                self.stack[self.sp as usize] = self.res;
                self.flags = self.alu_flags;
                self.pc += 1;
            },
            Command::MUL => {
                self.sp -= 1;
                self.stack[self.sp as usize] = self.res;
                self.flags = self.alu_flags;
                self.pc += 1;
            }
            Command::INC => {
                self.stack[self.sp as usize] = self.res;
                self.flags = self.alu_flags;
                self.pc += 1;
            },
            Command::READ => {
                self.stack[self.sp as usize] = self.res;
                self.pc += 1;
            },
            Command::LDC => {
                self.cnt = self.op1;
                self.sp -= 1;
                self.pc += 1;
            },
            Command::LOOP => {
                self.cnt = self.res;
                if self.res != 0 {
                    self.pc = self.addr;
                } else {
                    self.pc += 2;
                }
            },
            Command::NOP => {
                self.pc += 1;
            },
            Command::HALT => self.halt = true,
        }
    }

    pub fn deblog(&self) {
        println!("{:?}", self.stack);
        println!("sp: {}, pc: {}, op1: {}, op2: {}, op3: {}", self.sp, self.pc, self.op1, self.op2, self.op3);
        println!("cmd: {:?}", self.cmd);
        println!("state: {:?}", self.state);
    }
}
