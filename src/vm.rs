use crate::inst::{MASK_ADDI, MASK_LB, MATCH_ADDI, MATCH_LB};

#[derive(Default)]
pub struct VM {
    memory: Vec<u8>,
    registers: [i32; 32],
    pc: usize,
}

impl VM {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            pc: 0,
            memory,
            registers: [0; 32],
        }
    }

    pub fn run(&mut self) {
        self.handle_next_instruction();
    }

    fn handle_next_instruction(&mut self) {
        let pc = self.pc;

        assert!(pc + 4 <= self.memory.len(), "Unexpected end of program");

        let bytes = &self.memory[pc..pc + 4];
        let inst = u32::from_le_bytes(bytes.try_into().unwrap());

        if inst & MASK_ADDI == MATCH_ADDI {
            self.handle_i_instruction(inst, |_, rs1_val, imm| rs1_val.wrapping_add(imm));
        } else if inst & MASK_LB == MATCH_LB {
            self.handle_i_instruction(inst, |mem, rs1_val, imm| {
                let addr = rs1_val.wrapping_add(imm) as usize;
                let byte = *mem.get(addr).expect("Memory access out of bounds");
                (byte as i8) as i32
            });
        }

        self.pc += 4;
    }

    fn handle_i_instruction<F>(&mut self, instruction: u32, operation: F)
    where
        F: FnOnce(&mut Vec<u8>, i32, i32) -> i32,
    {
        let rd = ((instruction >> 7) & 0x1f) as usize;
        let rs1 = ((instruction >> 15) & 0x1f) as usize;
        let imm = (instruction as i32) >> 20;

        let rs1_val = self.registers[rs1];
        let result = operation(&mut self.memory, rs1_val, imm);

        if rd != 0 {
            self.registers[rd] = result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VM;

    #[test]
    fn test_addi() {
        // ADDI x1, x0, 5
        // 000000000101 00000 000 00001 0010011
        let mut vm = VM::new(vec![0x93, 0x00, 0x50, 0x00]);
        vm.run();
        assert_eq!(vm.registers[1], 5);
    }

    #[test]
    fn test_lb() {
        // LB x1, 4(x0)
        // 000000000100 00000 000 00001 0000011
        let mut vm = VM::new(vec![0x83, 0x00, 0x40, 0x00, 0x05]);
        vm.run();
        assert_eq!(vm.registers[1], 0x05);
    }
}
