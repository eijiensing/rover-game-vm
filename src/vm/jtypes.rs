use crate::inst::{MASK_JAL, MATCH_JAL};

use super::common::{InstructionDefinition, Opcode, OperandsFormat, EXMEM, IDEX};

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

pub const JTYPE_LIST: [InstructionDefinition; 1] = [
    InstructionDefinition {
        mask: MASK_JAL,
        match_val:MATCH_JAL,
        decode: |instruction, _| { 
            IDEX {
                opcode: Opcode::Jal,
                operands: Some(extract_jtype(instruction)),
                memory_operation: None,
            }
        },
        execute: |id_ex, pc| {
            if let Some(OperandsFormat::Jtype { rd, imm }) = &id_ex.operands {
                let old_pc = *pc - 4;
                *pc = old_pc.wrapping_add(*imm as usize);
                EXMEM {
                    rd: Some(*rd),
                    calculation_result: old_pc.wrapping_add(4) as i32,
                    memory_operation: None,
                    operands: id_ex.operands.clone(),
                }
            } else {
                unreachable!()
            }
        },
        opcode: Opcode::Jal
    },
];

