use crate::inst::{MASK_ADD, MASK_ADDI, MATCH_ADD, MATCH_ADDI};

use super::common::{InstructionDefinition, Opcode, OperandsFormat, EXMEM, IDEX};

fn extract_rtype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let rs1 = ((instruction >> 15) & 0x1f) as usize;
    let rs2 = ((instruction >> 20) & 0x1f) as usize;

    let rs1_value = registers[rs1];
    let rs2_value = registers[rs2];

    OperandsFormat::Rtype {
        rd: ((instruction >> 7) & 0x1f) as usize,
        r1_val: rs1_value,
        r2_val: rs2_value,
    }
}


pub const RTYPE_LIST: [InstructionDefinition; 1] = [
    InstructionDefinition {
        mask: MASK_ADD,
        match_val:MATCH_ADD,
        decode: |instruction, registers| { 
            IDEX {
                opcode: Opcode::Add,
                operands: Some(extract_rtype(instruction, registers)),
                memory_operation: None,
            }
        },
        execute: |id_ex, _| {
            if let Some(OperandsFormat::Rtype { rd, r1_val, r2_val }) = &id_ex.operands {
                EXMEM {
                    rd: Some(*rd),
                    calculation_result: r1_val.wrapping_add(*r2_val),
                    memory_operation: None,
                    operands: id_ex.operands.clone(),
                }
            } else {
                unreachable!()
            }
        },
        opcode: Opcode::Add
    },
];
