use std::{
    env, 
    fs::File, 
    io::{
        BufRead, BufReader, Write,
    }, 
    process
};

#[derive(Debug)]
#[allow(dead_code)]
enum Command {
    PUSH(u8),
    LOOP(u8),
    POP,
    DUP,
    READ,
    SWAP,
    ROT,
    LDC,
    ADD,
    INC,
    MUL,
    NOP,
    DATA(Vec<u8>),
    ADDR
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Использование: {} <путь_к_файлу>", args[0]);
        process::exit(1);
    }
    let path = &args[1];

    let mut prog: Vec<u8> = Vec::new();
    let mut mem: Vec<u8> = Vec::new();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let cmd = parse(&line);
        insertmem(&cmd, &mut prog, &mut mem);
    }

    let mut firmware: Vec<u8> = [0xff; 512].to_vec();
    let mut file = File::create("firmware.bin").unwrap();
    for indx in 0..prog.len() {
        firmware[indx] = prog[indx];
    }
    for indx in 384..mem.len() {
        firmware[indx] = mem[indx - 384];
    }

    file.write(&firmware).unwrap();
}

fn parse(line: &String) -> Command {
    let line: Vec<&str> = line.split(' ').collect();

    match line[0] {
        "PUSH" => Command::PUSH(line[1].parse::<u8>().unwrap()),
        "LOOP" => Command::LOOP(line[1].parse::<u8>().unwrap()),
        "POP" => Command::POP,
        "DUP" => Command::DUP,
        "READ" => Command::READ,
        "SWAP" => Command::SWAP,
        "ROT" => Command::ROT,
        "LDC" => Command::LDC,
        "ADD" => Command::ADD,
        "INC" => Command::INC,
        "MUL" => Command::MUL,
        "NOP" => Command::NOP,
        ".data" => Command::DATA(vec![]), /* TODO */
        ".p" => Command::ADDR, /* TODO */
        _ => panic!("Unknown command {}", line[0])
    }
}

fn insertmem(cmd: &Command, prog: &mut Vec<u8>, mem: &mut Vec<u8>) {
    match cmd {
        Command::PUSH(val) => {
            prog.push(0); 
            prog.push(*val)
        },
        Command::LOOP(addr) => {
            prog.push(6);
            prog.push(*addr);
        }
        Command::POP => prog.push(1),
        Command::SWAP => prog.push(2),
        Command::ROT => prog.push(3),
        Command::DUP => prog.push(4),
        Command::LDC => prog.push(5),
        Command::ADD => prog.push(7),
        Command::INC => prog.push(8),
        Command::MUL => prog.push(9),
        Command::READ => prog.push(10),
        Command::NOP => prog.push(0xff),
        
        Command::DATA(mas) => mem.extend(mas),
        Command::ADDR => panic!(""),
    }
}
