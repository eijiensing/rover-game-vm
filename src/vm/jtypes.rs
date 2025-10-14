use crate::inst::{MASK_JAL, MATCH_JAL};

use super::common::{EXMEM, ExecuteResult, IDEX, InstructionDefinition, Opcode, OperandsFormat};

fn extract_jtype(instruction: u32) -> OperandsFormat {
    let imm20 = ((instruction >> 31) & 0x1) << 20;
    let imm10_1 = ((instruction >> 21) & 0x3ff) << 1;
    let imm11 = ((instruction >> 20) & 0x1) << 11;
    let imm19_12 = ((instruction >> 12) & 0xff) << 12;

    let raw_imm = imm20 | imm19_12 | imm11 | imm10_1;

    // raw_imm already contains bits [20:1]; bit-0 is implicitly zero
    let imm = ((raw_imm << 1) as i32) >> 1;

    OperandsFormat::Jtype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        imm,
    }
}

pub const JTYPE_LIST: [InstructionDefinition; 1] = [InstructionDefinition {
    mask: MASK_JAL,
    match_val: MATCH_JAL,
    decode: |instruction, _, address| IDEX {
        opcode: Opcode::Jal,
        operands: Some(extract_jtype(instruction)),
        memory_operation: None,
        address,
    },
    execute: |id_ex| {
        if let Some(OperandsFormat::Jtype { rd, imm }) = &id_ex.operands {
            let old_pc = id_ex.address;
            let new_pc = Some(old_pc.wrapping_add(*imm as usize));
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
    opcode: Opcode::Jal,
}];
