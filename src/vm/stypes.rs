use crate::inst::{MASK_SB, MATCH_SB};

use super::common::{InstructionDefinition, MemoryOperation, MemoryRange, Opcode, OperandsFormat, EXMEM, IDEX};

fn extract_stype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let rs1 = ((instruction >> 15) & 0x1f) as usize;
    let rs2 = ((instruction >> 20) & 0x1f) as usize;

    let rs1_value = registers[rs1];
    let rs2_value = registers[rs2];

    // S-type immediate is split between bits [31:25] and [11:7]
    let imm_11_5 = ((instruction >> 25) & 0x7f) as i32;
    let imm_4_0 = ((instruction >> 7) & 0x1f) as i32;
    let imm = (imm_11_5 << 5) | imm_4_0;

    // Sign-extend
    let imm = (imm << 20) >> 20;

    OperandsFormat::Stype {
        r1_val: rs1_value,
        r2_val: rs2_value,
        imm,
    }
}

pub const STYPE_LIST: [InstructionDefinition; 1] = [
    InstructionDefinition {
        mask: MASK_SB,
        match_val:MATCH_SB,
        decode: |instruction, registers| { 
            IDEX {
                opcode: Opcode::Sb,
                operands: Some(extract_stype(instruction, registers)),
                memory_operation: Some(MemoryOperation {
                    is_load: false,
                    memory_range: MemoryRange::Byte,
                }),
            }
        },
        execute: |id_ex, _| {
            if let Some(OperandsFormat::Stype { r1_val, imm, r2_val: _ }) = &id_ex.operands {
                EXMEM {
                    rd: None,
                    calculation_result: r1_val.wrapping_add(*imm),
                    memory_operation: id_ex.memory_operation.clone(),
                    operands: id_ex.operands.clone(),
                }
            } else {
                unreachable!()
            }
        },
        opcode: Opcode::Sb
    },
];
