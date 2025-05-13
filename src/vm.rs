use crate::inst::{MASK_ADDI, MASK_LB, MATCH_ADDI, MATCH_LB};

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
            self.handle_i_instruction(bytes_instruction, |_, a, b| a.wrapping_add(b));
        } else if bytes_instruction & MASK_LB == MATCH_LB {
            self.handle_i_instruction(bytes_instruction, |vm, a, b| {
                let addr = a.wrapping_add(b) as usize;
                let byte = vm.memory[addr]; // load 1 byte
                (byte as i8) as i32
            });
        }

        0
    }

    fn handle_i_instruction<F>(&mut self, bytes_instruction: u32, operation: F)
    where
        F: FnOnce(&mut Self, i32, i32) -> i32,
    {
        let rd = ((bytes_instruction >> 7) & 0x1f) as usize;
        let rs1 = ((bytes_instruction >> 15) & 0x1f) as usize;
        let imm = (bytes_instruction as i32) >> 20;

        let rs1_val = self.registers[rs1];
        let result = operation(self, rs1_val, imm);
        self.registers[rd] = result;
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
    fn test_addi() {
        // ADDI is a I type instruction
        // imm          rs1   f3  rd    opcode
        // 000000000101 00000 000 00000 0010011
        // 00000000 01010000 00000000 00010011
        // 0x13 0x0 0x50 0x0
        let mut vm = VM::new(0, vec![0x13, 0x0, 0x50, 0x0]);
        vm.run();
        panic!("hi");
    }
}
