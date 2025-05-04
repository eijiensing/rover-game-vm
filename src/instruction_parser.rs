use crate::vm::{Immediate, Instruction, Opcode, Register};

pub fn parse_line(line: &str) -> Option<Instruction> {
    let line = line.trim_start();
    let first_word = line
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    if first_word.is_empty() {
        return None;
    }

    let operands_string = &line[first_word.len()..].split(",").collect::<Vec<&str>>();

    println!("|{first_word}|");

    let xd = match first_word.as_str() {
        "li" => parse_register(),
        _ => todo!(),
    };

    println!("{xd:?}");

    Some(Instruction {
        energy_cost: 0,
        opcode: Opcode::Addition,
        debug: None,
    })
}

enum Operand {
    Immediate,
    Register,
}

pub fn parse_operands(patterns: Vec<Operand>, parts: Vec<&str>) -> Option<()> {
    if patterns.len() != parts.len() {
        return None;
    }

    for (i, operand) in patterns.iter().enumerate() {
        match operand {
            Operand::Immediate => parse_immediate(parts.get(i).unwrap()),
            Operand::Register => parse_register(parts.get(i).unwrap()),
        };
    }
    None
}

pub fn parse_immediate(line: &str) -> (Option<Immediate>, &str) {
    let line = line.trim_start();
    let number_string = line
        .chars()
        .take_while(|c| c.is_ascii_digit() || c == &'-')
        .collect::<String>();

    if number_string.is_empty() {
        return (None, line);
    }

    let number = number_string.parse::<i32>().unwrap();

    (Some(Immediate(number)), &line[number_string.len()..])
}

pub fn parse_register(line: &str) -> (Option<Register>, &str) {
    let line = line.trim_start();
    println!("line|{line}|");
    let first_word = line
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    println!("first_word|{first_word}|");
    if first_word.len() != 1 {
        return (None, line);
    }

    let number_string = line[1..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();

    println!("number_string|{number_string}|");
    if number_string.is_empty() {
        return (None, line);
    }

    let number = number_string.parse::<u8>().unwrap();

    match first_word.as_str() {
        "x" => (Some(Register(number)), &line[1 + number_string.len()..]),
        "t" => (Some(Register(number)), &line[1 + number_string.len()..]),
        _ => (None, line),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_line() {
        let result = parse_line("   li t0, -1");
        assert!(result.is_none());
    }
}
