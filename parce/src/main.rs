use std::{
    io::{
        self,
        BufRead,
        BufReader,
    },
    fs::File,
    process,
    env
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
        println!("{:?}", parse(&line))
    }
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
