use super::{
    btypes::BTYPE_LIST,
    common::{EXMEM, IDEX, IFID, InstructionDefinition, MEMWB, MemoryRange, OperandsFormat},
    itypes::ITYPE_LIST,
    jtypes::JTYPE_LIST,
    rtypes::RTYPE_LIST,
    stypes::STYPE_LIST,
    utypes::UTYPE_LIST,
};

pub enum HazardAction {
    None,
    Stall,
    ForwardExecute(bool), // boolean indicates if it concerns the first register like r1 or r2
    ForwardMemory(bool),  // boolean indicates if it concerns the first register like r1 or r2
}

#[derive(Default)]
pub struct VM {
    instruction_definitions: Vec<InstructionDefinition>,
    memory: Vec<u8>,
    registers: [i32; 32],
    pc: usize,
    cycle: usize,
    stall: bool,
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
        ]
        .concat();

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
            stall: false,
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

    pub fn run(&mut self) {
        loop {
            self.step();
            if self.if_id.is_none()
                && self.id_ex.is_none()
                && self.ex_mem.is_none()
                && self.mem_wb.is_none()
            {
                break;
            }
        }
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
        if self.stall {
            return;
        }
        let pc = self.pc;
        // assert!(pc + 4 <= self.memory.len(), "Unexpected end of program");
        if pc + 4 <= self.memory.len() {
            let bytes = &self.memory[pc..pc + 4];
            let instruction = u32::from_le_bytes(bytes.try_into().unwrap());
            self.if_id = Some(IFID {
                instruction,
                address: self.pc,
            });
            // eagerly update the pc, this can be overwritten in the execute stage if the instruction
            // is a branch/ jump
            self.pc += 4;
        } else {
            self.if_id = None;
        }
    }

    fn decode(&mut self) {
        self.stall = false;

        let Some(if_id) = self.if_id.as_ref() else {
            self.id_ex = None;
            return;
        };

        for def in &self.instruction_definitions {
            if if_id.instruction & def.mask == def.match_val {
                let mut decoded = (def.decode)(if_id.instruction, &self.registers, if_id.address);

                match self.detect_data_hazard(&decoded) {
                    HazardAction::ForwardExecute(target_r1) => {
                        if let Some(ex_mem) = &self.ex_mem {
                            match decoded.operands.as_mut() {
                                Some(
                                    OperandsFormat::Btype { r1_val, r2_val, .. }
                                    | OperandsFormat::Stype { r1_val, r2_val, .. }
                                    | OperandsFormat::Rtype { r1_val, r2_val, .. },
                                ) => {
                                    if target_r1 {
                                        // if true forward result to r1
                                        *r1_val = ex_mem.calculation_result;
                                    } else {
                                        *r2_val = ex_mem.calculation_result;
                                    }
                                }
                                Some(OperandsFormat::Itype { r1_val, .. }) => {
                                    *r1_val = ex_mem.calculation_result;
                                }
                                _ => (),
                            }
                        }
                        self.id_ex = Some(decoded);
                    }
                    HazardAction::ForwardMemory(target_r1) => {
                        if let Some(mem_wb) = &self.mem_wb {
                            match decoded.operands.as_mut() {
                                Some(
                                    OperandsFormat::Btype { r1_val, r2_val, .. }
                                    | OperandsFormat::Stype { r1_val, r2_val, .. }
                                    | OperandsFormat::Rtype { r1_val, r2_val, .. },
                                ) => {
                                    if target_r1 {
                                        // if true forward result to r1
                                        *r1_val = mem_wb.value;
                                    } else {
                                        *r2_val = mem_wb.value;
                                    }
                                }
                                Some(OperandsFormat::Itype { r1_val, .. }) => {
                                    *r1_val = mem_wb.value;
                                }
                                _ => (),
                            }
                        }
                        self.id_ex = Some(decoded);
                    }
                    HazardAction::None => {
                        self.id_ex = Some(decoded);
                    }
                    HazardAction::Stall => {
                        self.id_ex = None;
                        self.stall = true;
                    }
                }
                break;
            }
        }
    }

    fn execute(&mut self) {
        let id_ex = match self.id_ex.as_ref() {
            Some(v) => v,
            None => {
                self.ex_mem = None;
                return;
            }
        };

        let result = (id_ex.execute)(id_ex);

        if let Some(new_pc) = result.new_pc {
            self.pc = new_pc;
        }

        if result.flush {
            self.if_id = None;
            self.id_ex = None;
        }

        self.ex_mem = Some(result.ex_mem);
    }

    fn memory(&mut self) {
        let Some(ex_mem) = self.ex_mem.take() else {
            self.mem_wb = None;
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

        if mem_wb.rd != 0 {
            // x0 must always be 0
            self.registers[mem_wb.rd] = mem_wb.value;
        }
    }

    fn detect_data_hazard(&self, id_ex: &IDEX) -> HazardAction {
        match id_ex.operands {
            Some(OperandsFormat::Rtype { r1, r2, .. }) => self.check_steps(&[r1, r2]),
            Some(OperandsFormat::Itype { r1, .. }) => self.check_steps(&[r1]),
            Some(OperandsFormat::Stype { r1, r2, .. }) => self.check_steps(&[r1, r2]),
            Some(OperandsFormat::Btype { r1, r2, .. }) => self.check_steps(&[r1, r2]),
            _ => HazardAction::None,
        }
    }

    fn check_steps(&self, registers: &[usize]) -> HazardAction {
        if let Some(ex_mem) = &self.ex_mem {
            if let Some(rd) = ex_mem.rd {
                if rd != 0 {
                    let maybe_rd_index = registers.iter().position(|y| y == &rd);
                    if let Some(rd_index) = maybe_rd_index {
                        if ex_mem.memory_operation.as_ref().is_some_and(|x| x.is_load) {
                            return HazardAction::Stall;
                        }
                        return HazardAction::ForwardExecute(rd_index == 0);
                    }
                }
            }
        }
        if let Some(mem_wb) = &self.mem_wb {
            let maybe_rd_index = registers.iter().position(|y| y == &mem_wb.rd);
            if let Some(rd_index) = maybe_rd_index {
                return HazardAction::ForwardMemory(rd_index == 0);
            }
        }
        HazardAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::VM;

    // === DATA HAZARDS ==============

    #[test]
    fn test_data_hazard_addi() {
        // ADDI x8, x0, 5
        // ADDI x9, x8, 5
        let mut vm = VM::new(vec![0x13, 0x04, 0x50, 0x00, 0x93, 0x04, 0x54, 0x00]);
        vm.run();
        assert_eq!(vm.registers[8], 5);
        assert_eq!(vm.registers[9], 10);
        assert_eq!(vm.cycle, 6);
    }

    #[test]
    fn test_data_hazard_lb() {
        // LB x8, 8(x0)
        // ADDI x9, x8, 5
        let mut vm = VM::new(vec![0x03, 0x04, 0x80, 0x00, 0x93, 0x04, 0x54, 0x00, 0x05]);
        vm.run();
        assert_eq!(vm.registers[8], 5);
        assert_eq!(vm.registers[9], 10);
        assert_eq!(vm.cycle, 7);
    }

    #[test]
    fn test_data_hazard_lb_addi() {
        // LB x8, 12(x0)
        // ADDI x9, x8, 5
        // ADDI x10, x9, 5
        let mut vm = VM::new(vec![
            0x03, 0x04, 0xc0, 0x00, 0x93, 0x04, 0x54, 0x00, 0x13, 0x85, 0x54, 0x00, 0x05,
        ]);
        vm.run();
        assert_eq!(vm.registers[8], 5);
        assert_eq!(vm.registers[9], 10);
        assert_eq!(vm.registers[10], 15);
        assert_eq!(vm.cycle, 8);
    }

    #[test]
    fn test_data_hazard_lb_sb() {
        // LB x8, 8(x0)
        // SB x8, 9(x0)
        let mut vm = VM::new(vec![
            0x03, 0x04, 0x80, 0x00, 0xa3, 0x04, 0x80, 0x00, 0x01, 0x02,
        ]);
        vm.registers[8] = 5;
        vm.run();
        assert_eq!(vm.registers[8], 0x01);
        assert_eq!(vm.memory[8], 0x01);
        assert_eq!(vm.cycle, 6); // 1 stall
    }

    #[test]
    fn test_data_hazard_addi_beq() {
        // ADDI x1, x0, 1
        // BEQ x1, x2, 8
        // ADDI x3, x0, 5 skip this one
        // ADDI x4, x0, 5

        let mut vm = VM::new(vec![
            0x93, 0x00, 0x10, 0x00, 0x63, 0x84, 0x20, 0x00, 0x93, 0x01, 0x50, 0x00, 0x13, 0x02,
            0x50, 0x00,
        ]);
        vm.registers[2] = 1;
        vm.run();

        assert_eq!(vm.registers[3], 0); // skipped
        assert_eq!(vm.registers[4], 5);
    }

    // === PIPELINED ==============

    #[test]
    fn test_bne_for_loop() {
        // ADDI x9, x0, 10
        // ADDI x8, x8, 1
        // BNE x8, x9, -4

        let program = vec![
            0x93, 0x04, 0xa0, 0x00, 0x13, 0x04, 0x14, 0x00, 0xe3, 0x1e, 0x94, 0xfe,
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.registers[8], 10);
    }

    #[test]
    fn test_jal_flush() {
        // JAL x1, 8
        // ADDI x2, x0, 42
        // ADDI x3, x0, 99

        let program = vec![
            0xef, 0x00, 0x80, 0x00, // JAL x1, 8
            0x13, 0x01, 0xa0, 0x02, // ADDI x2, x0, 42 (should be skipped)
            0x93, 0x01, 0x30, 0x06, // ADDI x3, x0, 99
        ];

        let mut vm = VM::new(program);
        vm.run();

        assert_eq!(vm.registers[1], 4); // x1 = return address (pc + 4 before jump)
        assert_eq!(vm.registers[2], 0); // x2 not set (skipped)
        assert_eq!(vm.registers[3], 99); // x3 set by ADDI
    }

    #[test]
    fn test_cycle_count() {
        // ADDI x0 x0 0
        let mut vm = VM::new(vec![0x13, 0x00, 0x00, 0x00]);
        vm.run();
        assert_eq!(vm.cycle, 5);

        // ADDI x0 x0 0
        // ADDI x0 x0 0
        let mut vm = VM::new(vec![0x13, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00]);
        vm.run();
        assert_eq!(vm.cycle, 6);

        // ADDI x0 x0 0
        // ADDI x0 x0 0
        // ADDI x0 x0 0
        let mut vm = VM::new(vec![
            0x13, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00,
        ]);
        vm.run();
        assert_eq!(vm.cycle, 7);
    }

    #[test]
    fn test_lb_twice_same_register() {
        // LB x8, 8(x0)
        // LB x8, 9(x0)
        let mut vm = VM::new(vec![
            0x03, 0x04, 0x80, 0x00, 0x03, 0x04, 0x90, 0x00, 0x01, 0x02,
        ]);
        vm.run();
        assert_eq!(vm.registers[8], 0x02);
        assert_eq!(vm.cycle, 6); // no stalls
    }

    #[test]
    fn test_sb_lb_raw() {
        // read after write
        // SB x0, 8(x0)
        // LB x9, 8(x0)
        let mut vm = VM::new(vec![0x23, 0x04, 0x00, 0x00, 0x83, 0x04, 0x80, 0x00, 0x01]);
        vm.registers[9] = 5;
        vm.run();
        assert_eq!(vm.registers[9], 0x00);
        assert_eq!(vm.cycle, 6); // no stalls
    }

    // === NON PIPELINED ==============

    #[test]
    fn test_addi() {
        // ADDI x1, x0, 5
        // 000000000101 00000 000 00001 0010011
        let mut vm = VM::new(vec![0x93, 0x00, 0x50, 0x00]);
        vm.step_no_pipeline();
        assert_eq!(vm.registers[1], 5);
    }

    #[test]
    fn test_addi_x0_remain_0() {
        // ADDI x0, x0, 5
        let mut vm = VM::new(vec![0x13, 0x00, 0x50, 0x00]);
        vm.step_no_pipeline();
        assert_eq!(vm.registers[0], 0);
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
        let mut vm = VM::new(vec![0x23, 0x02, 0x00, 0x00, 0x05]);
        vm.step_no_pipeline();
        assert_eq!(vm.memory[4], 0x00);
    }

    #[test]
    fn test_jal() {
        // JAL x1, 8
        // ADDI x2, x0, 42
        // ADDI x3, x0, 99

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
        // ADD x1, x2, x3
        let mut vm = VM::new(vec![0xb3, 0x00, 0x31, 0x00]);
        vm.registers[2] = 1;
        vm.registers[3] = 2;
        vm.step_no_pipeline();
        assert_eq!(vm.registers[1], 3);
    }

    #[test]
    fn test_sub() {
        // SUB x8, x9, x10
        let mut vm = VM::new(vec![0x33, 0x84, 0xa4, 0x40]);
        vm.registers[9] = 2;
        vm.registers[10] = 1;
        vm.step_no_pipeline();
        assert_eq!(vm.registers[8], 1);
    }

    #[test]
    fn test_xor() {
        // XOR x8, x9, x10
        let mut vm = VM::new(vec![0x33, 0xc4, 0xa4, 0x00]);
        vm.registers[9] = 0b11111000;
        vm.registers[10] = 0b00011110;
        vm.step_no_pipeline();
        assert_eq!(vm.registers[8], 0b11100110);
    }

    #[test]
    fn test_or() {
        // OR x8, x9, x10
        // 00 a4 e4 33
        let mut vm = VM::new(vec![0x33, 0xe4, 0xa4, 0x00]);
        vm.registers[9] = 0b11111000;
        vm.registers[10] = 0b00011110;
        vm.step_no_pipeline();
        assert_eq!(vm.registers[8], 0b11111110);
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

    #[test]
    fn test_bne() {
        // BNE x0, x1, 8
        // ADDI x2, x0, 42
        // ADDI x3, x0, 99

        let program = vec![
            0x63, 0x14, 0x10, 0x00, 0x13, 0x01, 0xa0, 0x02, 0x93, 0x01, 0x30, 0x06,
        ];

        let mut vm = VM::new(program);
        vm.registers[1] = 1; // make the condition false
        vm.step_no_pipeline();
        vm.step_no_pipeline();
        assert_eq!(vm.registers[2], 0); // x2 not set (skipped)
        assert_eq!(vm.registers[3], 99); // x3 set by ADDI
    }
}
