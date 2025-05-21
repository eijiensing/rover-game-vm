use crate::inst::{MASK_ADDI, MASK_LB, MASK_SB, MATCH_ADDI, MATCH_LB, MATCH_SB};

enum Opcode {
    Addi,
    Lb,
    Sb,
}

#[derive(Clone)]
enum OperandsFormat {
    Rtype { rd: usize, r1_val: i32, r2_val: i32 },
    Itype { rd: usize, r1_val: i32, imm: i32 },
    Stype { r1_val: i32, r2_val: i32, imm: i32 },
    Btype { r1_val: i32, r2_val: i32, imm: i32 },
    Utype { rd: usize, imm: i32 },
    Jtype { rd: usize, imm: i32 },
}

fn extract_itype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let rs1 = ((instruction >> 15) & 0x1f) as usize;

    let rs1_value = registers[rs1];

    OperandsFormat::Itype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        r1_val: rs1_value,
        imm: (instruction as i32) >> 20,
    }
}

fn extract_stype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let rs1 = ((instruction >> 15) & 0x1f) as usize;
    let rs2 = ((instruction >> 20) & 0x1f) as usize;

    let rs1_value = registers[rs1];
    let rs2_value = registers[rs2];

    // S-type immediate is split between bits [31:25] and [11:7]
    let imm_11_5 = ((instruction >> 25) & 0x7f) as i32;
    let imm_4_0 = ((instruction >> 7) & 0x1f) as i32;
    let imm = (imm_11_5 << 5) | imm_4_0;

    // Sign-extend
    let imm = (imm << 20) >> 20;

    OperandsFormat::Stype {
        r1_val: rs1_value,
        r2_val: rs2_value,
        imm,
    }
}

#[derive(Clone)]
enum MemoryRange {
    Byte,
    ByteUnsigned,
    Half,
    HalfUnsigned,
    Word,
}

#[derive(Clone)]
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
    rd: Option<usize>,
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

    pub fn step_no_pipeline(&mut self) {
        self.fetch();
        self.decode();
        self.execute();
        self.memory();
        self.writeback();
        self.cycle += 5;
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
        } else if if_id.instruction & MASK_LB == MATCH_LB {
            self.id_ex = Some(IDEX {
                opcode: Opcode::Lb,
                operands: Some(extract_itype(if_id.instruction, &self.registers)),
                memory_operation: Some(MemoryOperation {
                    is_load: true,
                    memory_range: MemoryRange::Byte,
                }),
            })
        } else if if_id.instruction & MASK_SB == MATCH_SB {
            self.id_ex = Some(IDEX {
                opcode: Opcode::Sb,
                operands: Some(extract_stype(if_id.instruction, &self.registers)),
                memory_operation: Some(MemoryOperation {
                    is_load: false,
                    memory_range: MemoryRange::Byte,
                }),
            })
        }
    }

    fn execute(&mut self) {
        let id_ex = match self.id_ex.as_ref() {
            Some(v) => v,
            None => return,
        };

        match (&id_ex.opcode, &id_ex.operands) {
            (
                Opcode::Addi,
                Some(OperandsFormat::Itype {
                    rd,
                    r1_val: rs1_value,
                    imm,
                }),
            ) => {
                self.ex_mem = Some(EXMEM {
                    rd: Some(*rd),
                    calculation_result: rs1_value.wrapping_add(*imm),
                    memory_operation: None,
                    operands: id_ex.operands.clone(),
                });
            }
            (
                Opcode::Lb,
                Some(OperandsFormat::Itype {
                    rd,
                    r1_val: rs1_value,
                    imm,
                }),
            ) => {
                self.ex_mem = Some(EXMEM {
                    rd: Some(*rd),
                    calculation_result: rs1_value.wrapping_add(*imm),
                    memory_operation: id_ex.memory_operation.clone(),
                    operands: id_ex.operands.clone(),
                });
            }
            (
                Opcode::Sb,
                Some(OperandsFormat::Stype {
                    r1_val,
                    r2_val: _,
                    imm,
                }),
            ) => {
                self.ex_mem = Some(EXMEM {
                    rd: None,
                    calculation_result: r1_val.wrapping_add(*imm),
                    memory_operation: id_ex.memory_operation.clone(),
                    operands: id_ex.operands.clone(),
                });
            }
            _ => panic!("Mismatched opcode and operand format"),
        }
    }

    fn memory(&mut self) {
        let Some(ex_mem) = self.ex_mem.take() else {
            return;
        };

        let addr = ex_mem.calculation_result as usize;
        let mut value = ex_mem.calculation_result;

        if let Some(mem_op) = &ex_mem.memory_operation {
            if mem_op.is_load {
                value = self.load_memory(mem_op.memory_range.clone(), addr);
            } else if let Some(OperandsFormat::Stype { r2_val, .. }) = &ex_mem.operands {
                self.store_memory(mem_op.memory_range.clone(), addr, *r2_val);
            }
        }

        if let Some(rd) = ex_mem.rd {
            self.mem_wb = Some(MEMWB { rd, value });
        }
    }

    fn load_memory(&self, kind: MemoryRange, addr: usize) -> i32 {
        match kind {
            MemoryRange::Byte => self.memory.get(addr).map_or(0, |&b| b as i8 as i32),
            MemoryRange::ByteUnsigned => self.memory.get(addr).map_or(0, |&b| b as i32),
            MemoryRange::Half => self.memory.get(addr..addr + 2).map_or(0, |bytes| {
                let half = u16::from_le_bytes(bytes.try_into().unwrap());
                half as i16 as i32
            }),
            MemoryRange::HalfUnsigned => self.memory.get(addr..addr + 2).map_or(0, |bytes| {
                u16::from_le_bytes(bytes.try_into().unwrap()) as i32
            }),
            MemoryRange::Word => self
                .memory
                .get(addr..addr + 4)
                .map_or(0, |bytes| i32::from_le_bytes(bytes.try_into().unwrap())),
        }
    }

    fn store_memory(&mut self, kind: MemoryRange, addr: usize, value: i32) {
        match kind {
            MemoryRange::Byte | MemoryRange::ByteUnsigned => {
                self.memory[addr] = value as u8;
            }
            MemoryRange::Half | MemoryRange::HalfUnsigned => {
                let bytes = (value as u16).to_le_bytes();
                self.memory[addr..addr + 2].copy_from_slice(&bytes);
            }
            MemoryRange::Word => {
                let bytes = value.to_le_bytes();
                self.memory[addr..addr + 4].copy_from_slice(&bytes);
            }
        }
    }
    fn writeback(&mut self) {
        let mem_wb = match self.mem_wb.as_ref() {
            Some(v) => v,
            None => return,
        };

        self.registers[mem_wb.rd] = mem_wb.value;
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
        vm.step_no_pipeline();
        assert_eq!(vm.registers[1], 5);
    }

    #[test]
    fn test_lb() {
        // LB x1, 4(x0)
        // 000000000100 00000 000 00001 0000011
        let mut vm = VM::new(vec![0x83, 0x00, 0x40, 0x00, 0x05]);
        vm.step_no_pipeline();
        assert_eq!(vm.registers[1], 0x05);
    }

    #[test]
    fn test_sb() {
        // SB x0, 4(x0)
        // 00000000 0000 00000 000 00100 0100011
        // 00000000 00000000 00000010 00100011
        let mut vm = VM::new(vec![0x23, 0x02, 0x00, 0x00, 0x05]);
        vm.step_no_pipeline();
        assert_eq!(vm.memory[4], 0x00);
    }
}
