use crate::inst::{MASK_BEQ, MATCH_BEQ};

use super::common::{EXMEM, IDEX, InstructionDefinition, Opcode, OperandsFormat};

fn extract_btype(instruction: u32, registers: &[i32; 32]) -> OperandsFormat {
    let imm_11 = ((instruction >> 7) & 0x01) as i32;
    let imm_4_1 = ((instruction >> 8) & 0x0f) as i32;
    let imm_10_5 = ((instruction >> 25) & 0x3f) as i32;
    let imm_12 = ((instruction >> 31) & 0x01) as i32;

    let mut imm = (imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1);

    // Sign-extend 13-bit immediate
    if imm & (1 << 12) != 0 {
        imm |= !0 << 13;
    }

    let r1 = ((instruction >> 15) & 0x1f) as usize;
    let r2 = ((instruction >> 20) & 0x1f) as usize;

    OperandsFormat::Btype {
        imm,
        r1_val: registers[r1],
        r2_val: registers[r2],
        r1,
        r2,
    }
}

pub const BTYPE_LIST: [InstructionDefinition; 1] = [InstructionDefinition {
    opcode: Opcode::Beq,
    mask: MASK_BEQ,
    match_val: MATCH_BEQ,
    decode: |instruction, registers| IDEX {
        opcode: Opcode::Beq,
        operands: Some(extract_btype(instruction, registers)),
        memory_operation: None,
    },
    execute: |id_ex, pc| {
        if let Some(OperandsFormat::Btype {
            imm,
            r1_val,
            r2_val,
            r1: _,
            r2: _,
        }) = &id_ex.operands
        {
            let old_pc = *pc - 4;
            if r1_val == r2_val {
                *pc = old_pc.wrapping_add(*imm as usize);
            }
            EXMEM {
                rd: None,
                calculation_result: imm << 12,
                memory_operation: None,
                operands: id_ex.operands.clone(),
            }
        } else {
            unreachable!()
        }
    },
}];
