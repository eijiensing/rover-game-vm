use std::collections::HashMap;

use super::{btypes::BTYPE_LIST, common::{InstructionDefinition, MemoryRange, Opcode, OperandsFormat, EXMEM, IDEX, IFID, MEMWB}, itypes::ITYPE_LIST, jtypes::JTYPE_LIST, rtypes::RTYPE_LIST, stypes::STYPE_LIST, utypes::UTYPE_LIST};

#[derive(Default)]
pub struct VM {
    instruction_definitions: Vec<InstructionDefinition>,
    execution_table: HashMap<Opcode, fn(&IDEX, &mut usize) -> EXMEM>,
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
        let instruction_definitions: Vec<InstructionDefinition> = [
            &RTYPE_LIST[..],
            &ITYPE_LIST[..],
            &STYPE_LIST[..],
            &BTYPE_LIST[..],
            &UTYPE_LIST[..],
            &JTYPE_LIST[..],
        ].concat();

        let mut execution_table = HashMap::new();

        for def in instruction_definitions.clone() {
            execution_table.insert(def.opcode, def.execute);
        }

        Self {
            pc: 0,
            memory,
            registers: [0; 32],
            cycle: 0,
            if_id: None,
            id_ex: None,
            ex_mem: None,
            mem_wb: None,
            instruction_definitions,
            execution_table
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
        // eagerly update the pc, this can be overwritten in the execute stage if the instruction
        // is a branch/ jump
        self.pc += 4;
    }

    fn decode(&mut self) {
        let Some(if_id) = self.if_id.as_ref() else { return };

        for def in &self.instruction_definitions {
            if if_id.instruction & def.mask == def.match_val {
                self.id_ex = Some((def.decode)(if_id.instruction, &self.registers));
                break;
            }
        }
    }

    fn execute(&mut self) {
        let id_ex = match self.id_ex.as_ref() {
            Some(v) => v,
            None => return,
        };

        if let Some(execute_function) = self.execution_table.get(&id_ex.opcode) {
            self.ex_mem = Some(execute_function(id_ex, &mut self.pc));
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
    use super::{VM};

    // #[test]
    // fn test_extract_jtype() {
    //     // JAL x1, 8
    //     // 0 0000000100 0 00000000 00001 1101111
    //     let instruction = 0b00000000100000000000000011101111;
    //     let result = extract_jtype(instruction);
    //     assert!(matches!(result, OperandsFormat::Jtype { rd: 1, imm: 8 }));
    // }

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
        let mut vm = VM::new(vec![0x23, 0x02, 0x00, 0x00, 0x05]);
        vm.step_no_pipeline();
        assert_eq!(vm.memory[4], 0x00);
    }

    #[test]
    fn test_jal() {
        // JAL x1, 8
        // 00000000 10000000 00000000 11101111
        // ADDI x2, x0, 42
        // 00000010 10100000 00000001 00010011
        // ADDI x3, x0, 99
        // 00000110 00110000 00000001 10010011

        let program = vec![
            0xef, 0x00, 0x80, 0x00, // JAL x1, 8
            0x13, 0x01, 0xa0, 0x02, // ADDI x2, x0, 42 (should be skipped)
            0x93, 0x01, 0x30, 0x06, // ADDI x3, x0, 99
        ];

        let mut vm = VM::new(program);
        vm.step_no_pipeline(); // JAL
        vm.step_no_pipeline(); // ADDI x3

        assert_eq!(vm.registers[1], 4); // x1 = return address (pc + 4 before jump)
        assert_eq!(vm.registers[2], 0); // x2 not set (skipped)
        assert_eq!(vm.registers[3], 99); // x3 set by ADDI
    }

    #[test]
    fn test_add() {
        // ADD x0, x1, x2
        let mut vm = VM::new(vec![0x33, 0x80, 0x20, 0x00]);
        vm.registers[1] = 1;
        vm.registers[2] = 2;
        vm.step_no_pipeline();
        assert_eq!(vm.registers[0], 3);
    }

    #[test]
    fn test_lui() {
        // LUI x1, 1
        let mut vm = VM::new(vec![0xb7, 0x10, 0x00, 0x00]);
        vm.step_no_pipeline();
        assert_eq!(vm.registers[1], 4096);
    }


    #[test]
    fn test_beq() {
        // BEQ x0, x0, 8
        // ADDI x2, x0, 42
        // ADDI x3, x0, 99

        let program = vec![
            0x63, 0x04, 0x00, 0x00, // BEQ x0, x0, 8 
            0x13, 0x01, 0xa0, 0x02, // ADDI x2, x0, 42 (should be skipped)
            0x93, 0x01, 0x30, 0x06, // ADDI x3, x0, 99
        ];

        let mut vm = VM::new(program);
        vm.step_no_pipeline(); // BEQ
        vm.step_no_pipeline(); // ADDI x3

        assert_eq!(vm.registers[2], 0); // x2 not set (skipped)
        assert_eq!(vm.registers[3], 99); // x3 set by ADDI

        // BEQ x0, x1, 8
        // ADDI x2, x0, 42
        // ADDI x3, x0, 99
        let program = vec![
            0x63, 0x04, 0x10, 0x00, // BEQ x0, x1, 8 
            0x13, 0x01, 0xa0, 0x02, // ADDI x2, x0, 42 (should NOT be skipped)
            0x93, 0x01, 0x30, 0x06, // ADDI x3, x0, 99
        ];

        let mut vm = VM::new(program);
        vm.registers[1] = 1;
        vm.step_no_pipeline(); // BEQ
        vm.step_no_pipeline(); // ADDI x3
        vm.step_no_pipeline(); // ADDI x3

        assert_eq!(vm.registers[2], 42); // x2 set by ADDI
        assert_eq!(vm.registers[3], 99); // x3 set by ADDI
    }
}
