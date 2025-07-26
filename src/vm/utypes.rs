use crate::inst::{MASK_LUI, MATCH_LUI};

use super::common::{InstructionDefinition, Opcode, OperandsFormat, EXMEM, IDEX};

fn extract_utype(instruction: u32) -> OperandsFormat {
    OperandsFormat::Utype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        imm: (instruction >> 12) as i32,
    }
}

pub const UTYPE_LIST: [InstructionDefinition; 1] = [
    InstructionDefinition {
        opcode: Opcode::Lui,
        mask: MASK_LUI,
        match_val: MATCH_LUI,
        decode: |instruction, _| { 
            IDEX {
                opcode: Opcode::Lui,
                operands: Some(extract_utype(instruction)),
                memory_operation: None,
            }
        },
        execute: |id_ex, _| {
            if let Some(OperandsFormat::Utype { rd, imm }) = &id_ex.operands {
                EXMEM {
                    rd: Some(*rd),
                    calculation_result: imm << 12,
                    memory_operation: None,
                    operands: id_ex.operands.clone(),
                }
            } else {
                unreachable!()
            }
        },
    },
];

