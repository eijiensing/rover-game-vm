use crate::inst::{MASK_ADD, MATCH_ADD};

use super::common::{EXMEM, ExecuteResult, IDEX, InstructionDefinition, Opcode, OperandsFormat};

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

pub const RTYPE_LIST: [InstructionDefinition; 1] = [InstructionDefinition {
    mask: MASK_ADD,
    match_val: MATCH_ADD,
    decode: |instruction, registers, address| IDEX {
        opcode: Opcode::Add,
        operands: Some(extract_rtype(instruction, registers)),
        memory_operation: None,
        address,
    },
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
            }
        } else {
            unreachable!()
        }
    },
    opcode: Opcode::Add,
}];
