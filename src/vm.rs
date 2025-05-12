use crate::inst::{MASK_ADDI, MATCH_ADDI};

#[derive(Default)]
pub struct VM {
    memory: Vec<u8>,
    registers: [i32; 32],
    pc: usize,
}

impl VM {
    pub fn new(begin_address: usize, memory: Vec<u8>) -> Self {
        Self {
            pc: begin_address,
            memory: memory,
            registers: [0; 32],
        }
    }

    pub fn run(&mut self) {
        let instr = self.fetch_instruction();
        println!("Fetched instruction: {:#010x}", instr);
    }

    fn fetch_instruction(&mut self) -> u32 {
        let pc = self.pc;

        assert!(pc + 4 <= self.memory.len(), "Unexpected end of program");

        let bytes = &self.memory[pc..pc + 4];
        let bytes_instruction = u32::from_le_bytes(bytes.try_into().unwrap());

        if bytes_instruction & MASK_ADDI == MATCH_ADDI {
            println!("ADDI INST");
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use super::VM;

    #[test]
    fn test_create_vm() {
        let mut vm = VM::new(0, vec![0]);
        vm.run();
    }

    #[test]
    fn test_adfds() {
        let mut vm = VM::new(0, vec![0xC, 0x8, 0, 0, 0, 0, 0, 0]);
        vm.run();
        panic!("hi");
    }
}
