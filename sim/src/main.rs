use core;
use std::{env, fs};

const PROG_SIZE: usize = 384;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "firmware.bin".to_string());
    let fw = fs::read(&path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(1);
    });
    if fw.len() < PROG_SIZE {
        eprintln!("{path}: файл короче {PROG_SIZE} байт");
        std::process::exit(1);
    }
    let prog: Vec<i8> = fw[..PROG_SIZE].iter().map(|&b| b as i8).collect();
    let mem: Vec<i8> = fw[PROG_SIZE..].iter().map(|&b| b as i8).collect();

    let mut core = core::Core::new();
    while !core.halt {
        core.step(&prog, &mem);
    }
}
