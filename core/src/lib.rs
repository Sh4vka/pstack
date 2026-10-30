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
    pub state: State
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
            state: State::FETCH
        }
    }
    
    pub fn step(&mut self, prog: &Vec<i8>, mem: &Vec<i8>) {
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
            Command::ADD => self.res = self.op1 + self.op2,
            Command::INC => self.res = self.op1 + 1,
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
                self.pc += 1;
            },
            Command::MUL => {
                self.pc += 1;
            }
            Command::INC => {
                self.stack[self.sp as usize] = self.res;
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
        }
    }

    pub fn deblog(&self) {
        println!("{:?}", self.stack);
        println!("sp: {}, pc: {}, op1: {}, op2: {}, op3: {}", self.sp, self.pc, self.op1, self.op2, self.op3);
        println!("cmd: {:?}", self.cmd);
        println!("state: {:?}", self.state);
    }
}