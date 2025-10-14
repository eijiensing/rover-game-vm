#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum Opcode {
    Add,
    Addi,
    Lb,
    Sb,
    Jal,
    Lui,
    Beq,
    Bne
}

#[derive(Debug, Clone)]
pub enum OperandsFormat {
    Rtype {
        rd: usize,
        r1: usize,
        r2: usize,
        r1_val: i32,
        r2_val: i32,
    },
    Itype {
        rd: usize,
        r1: usize,
        r1_val: i32,
        imm: i32,
    },
    Stype {
        r1: usize,
        r2: usize,
        r1_val: i32,
        r2_val: i32,
        imm: i32,
    },
    Btype {
        r1: usize,
        r2: usize,
        r1_val: i32,
        r2_val: i32,
        imm: i32,
    },
    Utype {
        rd: usize,
        imm: i32,
    },
    Jtype {
        rd: usize,
        imm: i32,
    },
}

#[derive(Clone)]
pub struct InstructionDefinition {
    pub mask: u32,
    pub match_val: u32,
    pub opcode: Opcode,
    pub decode: fn(u32, &[i32; 32], usize) -> IDEX,
    pub execute: fn(&IDEX) -> ExecuteResult,
}

#[derive(Debug, Clone)]
pub struct ExecuteResult {
    pub ex_mem: EXMEM,
    pub flush: bool,
    pub new_pc: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum MemoryRange {
    Byte,
    ByteUnsigned,
    Half,
    HalfUnsigned,
    Word,
}

#[derive(Debug, Clone)]
pub struct MemoryOperation {
    pub is_load: bool,
    pub memory_range: MemoryRange,
}

#[derive(Debug)]
pub struct IFID {
    pub instruction: u32,
    pub address: usize
}

#[derive(Debug)]
pub struct IDEX {
    pub opcode: Opcode,
    pub operands: Option<OperandsFormat>,
    pub memory_operation: Option<MemoryOperation>,
    pub address: usize
}

#[derive(Debug, Clone)]
pub struct EXMEM {
    pub rd: Option<usize>,
    pub calculation_result: i32,
    pub operands: Option<OperandsFormat>,
    pub memory_operation: Option<MemoryOperation>,
}

#[derive(Debug)]
pub struct MEMWB {
    pub rd: usize,
    pub value: i32,
}
