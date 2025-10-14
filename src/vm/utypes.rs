use crate::inst::{MASK_LUI, MATCH_LUI};

use super::common::{EXMEM, ExecuteResult, IDEX, InstructionDefinition, OperandsFormat};

fn extract_utype(instruction: u32) -> OperandsFormat {
    OperandsFormat::Utype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        imm: (instruction >> 12) as i32,
    }
}

pub const UTYPE_LIST: [InstructionDefinition; 1] = [InstructionDefinition {
    mask: MASK_LUI,
    match_val: MATCH_LUI,
    decode: |instruction, _, address| IDEX {
        operands: Some(extract_utype(instruction)),
        memory_operation: None,
        address,
        execute: |id_ex| {
            if let Some(OperandsFormat::Utype { rd, imm }) = &id_ex.operands {
                ExecuteResult {
                    ex_mem: EXMEM {
                        rd: Some(*rd),
                        calculation_result: imm << 12,
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
}];
