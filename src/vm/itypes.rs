use crate::inst::{MASK_ADDI, MASK_ANDI, MASK_LB, MASK_LBU, MASK_LH, MASK_LHU, MASK_LW, MASK_ORI, MASK_SLLI, MASK_SLTI, MASK_SLTIU, MASK_SRAI, MASK_SRLI, MASK_XORI, MATCH_ADDI, MATCH_ANDI, MATCH_LB, MATCH_LBU, MATCH_LH, MATCH_LHU, MATCH_LW, MATCH_ORI, MATCH_SLLI, MATCH_SLTI, MATCH_SLTIU, MATCH_SRAI, MATCH_SRLI, MATCH_XORI};

use super::common::{
    EXMEM, ExecuteResult, IDEX, InstructionDefinition, MemoryOperation, MemoryRange,
    OperandsFormat,
};

fn extract_itype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let r1 = ((instruction >> 15) & 0x1f) as usize;

    let rs1_value = registers[r1];

    OperandsFormat::Itype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        r1_val: rs1_value,
        imm: (instruction as i32) >> 20,
        r1,
    }
}

pub const ITYPE_LIST: [InstructionDefinition; 15] = [
    InstructionDefinition {
        mask: MASK_ADDI,
        match_val: MATCH_ADDI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_XORI,
        match_val: MATCH_XORI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val ^ *imm,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_ORI,
        match_val: MATCH_ORI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val | *imm,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_ANDI,
        match_val: MATCH_ANDI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val & *imm,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLLI,
        match_val: MATCH_SLLI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    let shamt = ((*imm) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: ((*r1_val as u32) << shamt) as i32,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SRLI,
        match_val: MATCH_SRLI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    let shamt = ((*imm) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: ((*r1_val as u32) >> shamt) as i32,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SRAI,
        match_val: MATCH_SRAI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    let shamt = ((*imm) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: *r1_val >> shamt,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLTI,
        match_val: MATCH_SLTI,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    let res = if *r1_val < *imm { 1 } else { 0 };
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: res,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLTIU,
        match_val: MATCH_SLTIU,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    let res = if (*r1_val as u32) < (*imm as u32) {
                        1
                    } else {
                        0
                    };
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: res,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LB,
        match_val: MATCH_LB,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::Byte,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: id_ex.memory_operation.clone(),
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LH,
        match_val: MATCH_LH,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::Half,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: id_ex.memory_operation.clone(),
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LW,
        match_val: MATCH_LW,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::Word,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: id_ex.memory_operation.clone(),
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LBU,
        match_val: MATCH_LBU,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::ByteUnsigned,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: id_ex.memory_operation.clone(),
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LHU,
        match_val: MATCH_LHU,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::HalfUnsigned,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*imm),
                            memory_operation: id_ex.memory_operation.clone(),
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_LHU,
        match_val: MATCH_LHU,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::HalfUnsigned,
            }),
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Itype {
                    rd, r1_val, imm, ..
                }) = &id_ex.operands
                {
                let old_pc = id_ex.address;
                let new_pc = Some(old_pc.wrapping_add((r1_val + imm) as usize));
                ExecuteResult {
                    ex_mem: EXMEM {
                        rd: Some(*rd),
                        calculation_result: old_pc.wrapping_add(4) as i32,
                        memory_operation: None,
                        operands: id_ex.operands.clone(),
                    },
                    flush: true,
                    new_pc,
                }
                } else {
                    unreachable!()
                }
            },
        },
    },
];
