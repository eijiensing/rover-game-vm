use crate::vm::{Immediate, Instruction, Opcode, Register};

pub fn parse_line(line: &str) -> Result<Instruction, AssemblerError> {
    let line = line.trim_start();
    let first_word = line
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    if first_word.is_empty() {
        return Err(AssemblerError::EmptyLine);
    }

    let operands_string = &line[first_word.len()..].split(",").collect::<Vec<&str>>();

    match first_word.as_str() {
        "li" => {
            let operands =
                parse_operands(vec![Operand::Register, Operand::Immediate], operands_string)?;

            let mut iter = operands.into_iter();

            let destination = match iter.next().unwrap() {
                ParsedValue::Register(r) => r,
                _ => unreachable!(),
            };

            let value = match iter.next().unwrap() {
                ParsedValue::Immediate(i) => i,
                _ => unreachable!(),
            };

            Ok(Instruction {
                energy_cost: 0,
                opcode: Opcode::LoadImmediate { destination, value },
                debug: None,
            })
        }
        _ => Err(AssemblerError::UnknownOpcode(first_word)),
    }
}

pub enum Operand {
    Immediate,
    Register,
}

pub fn parse_operands(
    patterns: Vec<Operand>,
    parts: &Vec<&str>,
) -> Result<Vec<ParsedValue>, AssemblerError> {
    if patterns.len() != parts.len() {
        return Err(AssemblerError::OperandMismatch {
            expected: patterns.len(),
            found: parts.len(),
        });
    }

    let mut results = Vec::new();

    for (i, operand) in patterns.iter().enumerate() {
        let part = parts.get(i).unwrap();
        let parsed = match operand {
            Operand::Immediate => parse_immediate(part).map(ParsedValue::Immediate),
            Operand::Register => parse_register(part).map(ParsedValue::Register),
        }?;
        results.push(parsed);
    }

    Ok(results)
}

#[derive(Debug)]
pub enum ParsedValue {
    Immediate(Immediate),
    Register(Register),
}

pub fn parse_immediate(line: &str) -> Result<Immediate, AssemblerError> {
    let line = line.trim_start();
    let number_string = line
        .chars()
        .take_while(|c| c.is_ascii_digit() || c == &'-')
        .collect::<String>();

    if number_string.is_empty() {
        return Err(AssemblerError::InvalidImmediate(line.into()));
    }

    let number = number_string
        .parse::<i32>()
        .map_err(|_| AssemblerError::InvalidImmediate(number_string.clone()))?;

    Ok(Immediate(number))
}

pub fn parse_register(line: &str) -> Result<Register, AssemblerError> {
    let line = line.trim_start();
    let first_word = line
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    if first_word.len() != 1 {
        return Err(AssemblerError::InvalidRegister(line.into()));
    }

    let number_string = line[1..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();

    if number_string.is_empty() {
        return Err(AssemblerError::InvalidRegister(number_string));
    }

    let number = number_string
        .parse::<u8>()
        .map_err(|_| AssemblerError::InvalidRegister(line.into()))?;

    match first_word.as_str() {
        "x" => Ok(Register(number)),
        "t" => Ok(Register(number)),
        _ => Err(AssemblerError::InvalidRegister(line.into())),
    }
}

#[derive(Debug)]
pub enum AssemblerError {
    EmptyLine,
    UnknownOpcode(String),
    OperandMismatch { expected: usize, found: usize },
    InvalidRegister(String),
    InvalidImmediate(String),
    General(String),
}

use std::fmt;

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssemblerError::EmptyLine => write!(f, "Encountered an empty or whitespace-only line."),
            AssemblerError::UnknownOpcode(opcode) => write!(f, "Unknown opcode: '{}'.", opcode),
            AssemblerError::OperandMismatch { expected, found } => write!(
                f,
                "Operand count mismatch: expected {}, found {}.",
                expected, found
            ),
            AssemblerError::InvalidRegister(reg) => {
                write!(f, "Invalid register: '{}'.", reg)
            }
            AssemblerError::InvalidImmediate(imm) => {
                write!(f, "Invalid immediate value: '{}'.", imm)
            }
            AssemblerError::General(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_line() {
        let result = parse_line("   li t0, -1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_line() {
        let result = parse_line("   ");
        assert!(matches!(result, Err(AssemblerError::EmptyLine)));
    }

    #[test]
    fn test_parse_unknown_opcode() {
        let result = parse_line("foo t0, -1");
        assert!(matches!(result, Err(AssemblerError::UnknownOpcode(_))));
    }

    #[test]
    fn test_operand_mismatch() {
        let result = parse_line("li t0");
        assert!(matches!(
            result,
            Err(AssemblerError::OperandMismatch { .. })
        ));
    }

    #[test]
    fn test_invalid_register() {
        let result = parse_line("li z0, 5"); // assuming only x and t are valid
        assert!(matches!(result, Err(AssemblerError::InvalidRegister(_))));
    }

    #[test]
    fn test_invalid_immediate() {
        let result = parse_line("li t0, notanumber");
        assert!(matches!(result, Err(AssemblerError::InvalidImmediate(_))));
    }
}
