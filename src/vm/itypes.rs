use crate::inst::{MASK_ADDI, MASK_LB, MATCH_ADDI, MATCH_LB};

use super::common::{
    EXMEM, ExecuteResult, IDEX, InstructionDefinition, MemoryOperation, MemoryRange, Opcode,
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

pub const ITYPE_LIST: [InstructionDefinition; 2] = [
    InstructionDefinition {
        opcode: Opcode::Addi,
        mask: MASK_ADDI,
        match_val: MATCH_ADDI,
        decode: |instruction, registers, address| IDEX {
            opcode: Opcode::Addi, 
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: None,
            address,
        },
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
    InstructionDefinition {
        mask: MASK_LB,
        match_val: MATCH_LB,
        decode: |instruction, registers, address| IDEX {
            opcode: Opcode::Lb,
            operands: Some(extract_itype(instruction, registers)),
            memory_operation: Some(MemoryOperation {
                is_load: true,
                memory_range: MemoryRange::Byte,
            }),
            address,
        },
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
        opcode: Opcode::Lb,
    },
];
