use std::{
    collections::HashMap, env, fs, process
};

const PROG_SIZE: usize = 384;
const FW_SIZE: usize = 512;

enum Operand {
    Num(u8),
    Sym(String),
}

struct Instr {
    line: usize,
    opcode: u8,
    operand: Option<Operand>,
}

enum Section {
    None,
    Data,
    Text
}

fn opcode(m: &str) -> Option<(u8, bool)> {
    Some(match m {
        "PUSH" => (0, true),
        "POP"  => (1, false),
        "SWAP" => (2, false),
        "ROT"  => (3, false),
        "DUP"  => (4, false),
        "LDC"  => (5, false),
        "LOOP" => (6, true),
        "ADD"  => (7, false),
        "INC"  => (8, false),
        "MUL"  => (9, false),
        "READ" => (10, false),
        "NOP"  => (0xff, false),
        "HALT" => (11, false),
        _ => return None,
    })
}

fn parse_num(s: &str, ln: usize) -> Result<u8, String> {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let v = if let Some(h) = body.strip_prefix("0x") {
        i16::from_str_radix(h, 16)
    } else {
        body.parse::<i16>()
    }
    .map_err(|_| format!("{ln}: плохое число '{s}'"))?;
    let v = if neg { -v } else { v };
    let v = i8::try_from(v).map_err(|_| format!("{ln}: число '{s}' вне диапазона -128..127"))?;
    Ok(v as u8)
}

fn assembly(src: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut symbols: HashMap<String, u8> = HashMap::new();
    let mut instr: Vec<Instr> = Vec::new();
    let mut mem: Vec<u8> = Vec::new();
    let mut addr: usize = 0;
    let mut section = Section::None;

    let mut define = |name: &str, val: usize, line: usize| -> Result<(), String> {
        let v = u8::try_from(val).map_err(|_| format!("{line}: Адрес {val} не влизает в байт"))?;
        if symbols.insert(name.to_string(), v).is_some() {
            return Err(format!("{line}: имя '{name}' уже определено"));
        }
        Ok(())
    };

    for (i, raw) in src.lines().enumerate() {
        let ln = i + 1;
        let text = raw.split("//").next().unwrap().split(";").next().unwrap().trim();
        if text.is_empty() {
            continue;
        }

        match text {
            ".data" => {section = Section::Data; continue;}
            ".text" => {section = Section::Text; continue;}
            _ => {}
        }

        match section {
            Section::None => return Err(format!("{ln}: сначала нужна секция .data или .text")),

            Section::Data => {
                let (name, val) = text.split_once('=').ok_or(format!("{ln}: Ожидалось ИМЯ = [...]"))?;
                let val = val.trim()
                    .strip_prefix('[').and_then(|v| v.strip_suffix(']'))
                    .ok_or(format!("Массив должен быть в []"))?;
                define(name.trim(), mem.len(), ln)?;
                for n in val.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    mem.push(parse_num(n, ln)?);
                }
            }

            Section::Text => {
                let mut text = text;
                if let Some((label, rest))= text.split_once(':') {
                    define(label.trim(), addr, ln)?;
                    text = rest.trim();
                    if text.is_empty() {continue;}
                }
                let mut parts = text.split_whitespace();
                let m = parts.next().unwrap().to_uppercase();
                let args = parts.next();
                if parts.next().is_some() {
                    return Err(format!("{ln}: лишнии токены"));
                }
                let (op, has_operand) = opcode(&m).ok_or(format!("{ln}: Неизвестная команда {m}"))?;
                let operand = match (has_operand, args) {
                    (true, Some(a)) => Some(
                        if a.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
                            Operand::Num(parse_num(a, ln)?)
                        } else {
                            Operand::Sym(a.to_string())
                        }
                    ),
                    (true, None) => return Err(format!("{ln}: {m} требуется операнд")),
                    (false, Some(_)) => return Err(format!("{ln}: {m} без операнда")),
                    (false, None) => None,
                };
                addr += if operand.is_some() {2} else {1};
                instr.push(Instr { line: ln, opcode: op, operand });
            }
        }
    }

    let mut prog = Vec::new();
    for ins in instr {
        prog.push(ins.opcode);
        match ins.operand {
            None => {},
            Some(Operand::Num(n)) => prog.push(n),
            Some(Operand::Sym(s)) => prog.push(
                *symbols.get(&s).ok_or(format!("{}: Неизвестное имя '{s}'", ins.line))?,
            ),
        }
    }
    if prog.len() > PROG_SIZE {
        return Err(format!("Программа {} байт, Максимум {PROG_SIZE}", prog.len()));
    }
    if mem.len() > FW_SIZE - PROG_SIZE {
        return Err("данные не влезают в память".into());
    }

    Ok((prog, mem))
}

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Использование: asmp <файл.as>");
        process::exit(1);
    });
    let src = fs::read_to_string(&path).unwrap_or_else(|e| { eprintln!("{path}: {e}"); process::exit(1) });
    let (prog, mem) = assembly(&src).unwrap_or_else(|e| { eprintln!("{path}:{e}"); process::exit(1) });

    let mut fw = vec![0xff_u8; FW_SIZE];
    fw[..prog.len()].copy_from_slice(&prog);
    fw[PROG_SIZE..PROG_SIZE + mem.len()].copy_from_slice(&mem);
    fs::write("firmware.bin", fw).unwrap();
}
