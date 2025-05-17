use crate::inst::{MASK_ADDI, MASK_LB, MATCH_ADDI, MATCH_LB};

enum Opcode {
    Addi,
}

enum OperandsFormat {
    Rtype {
        rd: usize,
        rs1_value: i32,
        rs2_value: i32,
    },
    Itype {
        rd: usize,
        rs1_value: i32,
        imm: i32,
    },
    Stype {
        rs1_value: i32,
        rs2_value: i32,
        imm: i32,
    },
    Btype {
        rs1_value: i32,
        rs2_value: i32,
        imm: i32,
    },
    Utype {
        rd: usize,
        imm: i32,
    },
    Jtype {
        rd: usize,
        imm: i32,
    },
}

fn extract_itype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let rs1 = ((instruction >> 15) & 0x1f) as usize;

    let rs1_value = registers[rs1];

    OperandsFormat::Itype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        rs1_value,
        imm: (instruction as i32) >> 20,
    }
}

enum MemoryRange {
    Byte,
    ByteUnsigned,
    Half,
    HalfUnsigned,
    Word,
}

struct MemoryOperation {
    is_load: bool,
    memory_range: MemoryRange,
}

struct IFID {
    instruction: u32,
}

struct IDEX {
    opcode: Opcode,
    operands: Option<OperandsFormat>,
    memory_operation: Option<MemoryOperation>,
}

struct EXMEM {
    calculation_result: i32,
    operands: Option<OperandsFormat>,
    memory_operation: Option<MemoryOperation>,
}

struct MEMWB {
    rd: usize,
    value: i32,
}

#[derive(Default)]
pub struct VM {
    memory: Vec<u8>,
    registers: [i32; 32],
    pc: usize,
    cycle: usize,
    if_id: Option<IFID>,
    id_ex: Option<IDEX>,
    ex_mem: Option<EXMEM>,
    mem_wb: Option<MEMWB>,
}

impl VM {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            pc: 0,
            memory,
            registers: [0; 32],
            cycle: 0,
            if_id: None,
            id_ex: None,
            ex_mem: None,
            mem_wb: None,
        }
    }

    pub fn run(&mut self) {
        self.handle_next_instruction();
    }

    pub fn step(&mut self) {
        self.writeback();
        self.memory();
        self.execute();
        self.decode();
        self.fetch();
        self.cycle += 1;
    }

    fn fetch(&mut self) {
        let pc = self.pc;
        assert!(pc + 4 <= self.memory.len(), "Unexpected end of program");
        let bytes = &self.memory[pc..pc + 4];
        let instruction = u32::from_le_bytes(bytes.try_into().unwrap());
        self.if_id = Some(IFID { instruction });
    }

    fn decode(&mut self) {
        let if_id = match self.if_id.as_ref() {
            Some(v) => v,
            None => return,
        };

        if if_id.instruction & MASK_ADDI == MATCH_ADDI {
            self.id_ex = Some(IDEX {
                opcode: Opcode::Addi,
                operands: Some(extract_itype(if_id.instruction, &self.registers)),
                memory_operation: None,
            });
        }
    }

    fn execute(&mut self) {
        let id_ex = match self.id_ex.as_ref() {
            Some(v) => v,
            None => return,
        };

        match (&id_ex.opcode, &id_ex.operands) {
            (Opcode::Addi, Some(OperandsFormat::Itype { rd, rs1_value, imm })) => {
                self.ex_mem = Some(EXMEM {
                    rd: *rd,
                    calculation_result: rs1_value.wrapping_add(*imm),
                    memory_operation: None,
                });
            }
            _ => panic!("Mismatched opcode and operand format"),
        }
    }
    fn memory(&mut self) {
        let ex_mem = match self.ex_mem.as_ref() {
            Some(v) => v,
            None => return,
        };

        let mut value = ex_mem.calculation_result;

        if let Some(mem_op) = ex_mem.memory_operation.as_ref() {
            if mem_op.is_load {
                let addr = ex_mem.calculation_result as usize;

                value = match mem_op.memory_range {
                    MemoryRange::Byte => {
                        let byte = *self.memory.get(addr).expect("Memory access out of bounds");
                        (byte as i8) as i32
                    }
                    MemoryRange::ByteUnsigned => {
                        let byte = *self.memory.get(addr).expect("Memory access out of bounds");
                        byte as i32
                    }
                    MemoryRange::Half => {
                        let bytes = self
                            .memory
                            .get(addr..addr + 2)
                            .expect("Memory access out of bounds");
                        let half = u16::from_le_bytes(bytes.try_into().unwrap());
                        (half as i16) as i32
                    }
                    MemoryRange::HalfUnsigned => {
                        let bytes = self
                            .memory
                            .get(addr..addr + 2)
                            .expect("Memory access out of bounds");
                        u16::from_le_bytes(bytes.try_into().unwrap()) as i32
                    }
                    MemoryRange::Word => {
                        let bytes = self
                            .memory
                            .get(addr..addr + 4)
                            .expect("Memory access out of bounds");
                        i32::from_le_bytes(bytes.try_into().unwrap())
                    }
                };
            }
        }

        self.mem_wb = Some(MEMWB {
            rd: ex_mem.rd,
            value,
        })
    }
    fn writeback(&mut self) {}

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
