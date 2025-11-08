use crate::inst::{
    MASK_ADD, MASK_AND, MASK_OR, MASK_SLL, MASK_SLT, MASK_SLTU, MASK_SRA, MASK_SRL, MASK_SUB, MASK_XOR, MATCH_ADD, MATCH_AND, MATCH_OR, MATCH_SLL, MATCH_SLT, MATCH_SLTU, MATCH_SRA, MATCH_SRL, MATCH_SUB, MATCH_XOR
};

use super::common::{EXMEM, ExecuteResult, IDEX, InstructionDefinition, OperandsFormat};

fn extract_rtype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let r1 = ((instruction >> 15) & 0x1f) as usize;
    let r2 = ((instruction >> 20) & 0x1f) as usize;

    let rs1_value = registers[r1];
    let rs2_value = registers[r2];

    OperandsFormat::Rtype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        r1_val: rs1_value,
        r2_val: rs2_value,
        r1,
        r2,
    }
}

pub const RTYPE_LIST: [InstructionDefinition; 10] = [
    InstructionDefinition {
        mask: MASK_ADD,
        match_val: MATCH_ADD,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_add(*r2_val),
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SUB,
        match_val: MATCH_SUB,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val.wrapping_sub(*r2_val),
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_XOR,
        match_val: MATCH_XOR,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val ^ *r2_val,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_OR,
        match_val: MATCH_OR,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val | *r2_val,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_AND,
        match_val: MATCH_AND,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: r1_val & *r2_val,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLL,
        match_val: MATCH_SLL,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    // bit range is always within 0..=31 so no panics
                    let shamt = ((*r2_val) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: ((*r1_val as u32) << shamt) as i32,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SRL,
        match_val: MATCH_SRL,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    let shamt = ((*r2_val) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: ((*r1_val as u32) >> shamt) as i32,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SRA,
        match_val: MATCH_SRA,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    let shamt = ((*r2_val) & 0x1f) as u32;
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: *r1_val >> shamt,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLT,
        match_val: MATCH_SLT,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    let res = if *r1_val < *r2_val { 1 } else { 0 };
                    ExecuteResult {
                        ex_mem: EXMEM {
                            rd: Some(*rd),
                            calculation_result: res,
                            memory_operation: None,
                            operands: id_ex.operands.clone(),
                        },
                        flush: false,
                        new_pc: None,
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
    InstructionDefinition {
        mask: MASK_SLTU,
        match_val: MATCH_SLTU,
        decode: |instruction, registers, address| IDEX {
            operands: Some(extract_rtype(instruction, registers)),
            memory_operation: None,
            address,
            execute: |id_ex| {
                if let Some(OperandsFormat::Rtype {
                    rd, r1_val, r2_val, ..
                }) = &id_ex.operands
                {
                    let res = if (*r1_val as u32) < (*r2_val as u32) {
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
                        trap_type: None,
                    }
                } else {
                    unreachable!()
                }
            },
        },
    },
];
