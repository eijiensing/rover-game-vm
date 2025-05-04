#[derive(Default)]
pub struct VM {
    pc: usize,
    registers: [i32; 32],
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Opcode {
    LoadImmediate {
        destination: Register,
        value: Immediate,
    },
    Addition,
}

#[derive(Debug)]
pub enum Operand {
    Register(u8),
    Immediate(i32),
}

#[derive(Debug)]
pub struct DebugInformation {
    line_number: u32,
}

#[derive(Debug)]
pub struct Instruction {
    pub energy_cost: u16,
    pub opcode: Opcode,
    pub debug: Option<DebugInformation>,
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) {
        while self.pc < self.instructions.len() {
            self.execute();
        }
    }

    pub fn execute(&mut self) {
        let instruction = self.instructions.get(self.pc).expect("Invalid pc!");

        match &instruction.opcode {
            Opcode::LoadImmediate { destination, value } => {
                self.registers[destination.0 as usize] = value.0
            }
            Opcode::Addition => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_vm() -> VM {
        VM::default()
    }

    #[test]
    fn opcode_load_immediate() {
        let mut vm = get_vm();
        vm.instructions = vec![Instruction {
            energy_cost: 0,
            opcode: Opcode::LoadImmediate {
                destination: Register(1),
                value: Immediate(10),
            },
            debug: None,
        }];

        vm.execute();

        let mut expected = [0; 32];
        expected[1] = 10;

        assert_eq!(vm.registers, expected);
    }
}
