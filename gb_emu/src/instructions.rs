use crate::{
    bus::MemoryAccessError,
    instruction_tables::{CBPREFIXED, UNPREFIXED},
};

pub struct Instruction {
    pub op_code: OpCode,
    pub operands: &'static [Operand],
    pub cycles: u8,
    pub bytes: u8,
    pub flags: FlagChecks,
}

impl Instruction {
    pub const fn new(op_code: OpCode, operands: &'static [Operand], cycles: u8, bytes: u8, flags: FlagChecks) -> Self {
        Self { op_code, operands, cycles, bytes, flags }
    }
}

impl TryFrom<[u8; 3]> for &Instruction {
    type Error = InstructionError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match UNPREFIXED[value[0] as usize].op_code {
            OpCode::Prefix => Ok(&CBPREFIXED[value[1] as usize]),
            OpCode::Illegal => Err(InstructionError::InvalidOperation(value[0])),
            _ => Ok(&UNPREFIXED[value[0] as usize]),
        }
    }
}
pub enum OpCode {
    Adc,
    Add,
    And,
    Bit,
    Call,
    Ccf,
    Cp,
    Cpl,
    Daa,
    Dec,
    Di,
    Ei,
    Halt,
    Illegal,
    Inc,
    Jp,
    Jr,
    Ld,
    Ldh,
    Nop,
    Or,
    Pop,
    Prefix,
    Push,
    Res,
    Ret,
    Reti,
    Rl,
    Rla,
    Rlc,
    Rlca,
    Rr,
    Rra,
    Rrc,
    Rrca,
    Rst,
    Sbc,
    Scf,
    Set,
    Sla,
    Sra,
    Srl,
    Stop,
    Sub,
    Swap,
    Xor,
    JpConditional,
    JrConditional,
    CallConditional,
    RetConditional,
}
pub enum Operand {
    A,
    A16,
    A16Pointer,
    AF,
    B,
    BC,
    BCPointer,
    C,
    D,
    DE,
    DEPointer,
    E,
    E8,
    FF00OffsetByA,
    FF00OffsetByC,
    H,
    HL,
    HLD,
    HLI,
    HLPointer,
    Immediate(u8),
    L,
    N16,
    N8,
    NC,
    NZ,
    SP,
    SPAndE8,
    Z,
}

pub struct FlagChecks {
    check_z: Option<FlagCheck>,
    set_n: Option<FlagCheck>,
    half_carry: Option<FlagCheck>,
    carry: Option<FlagCheck>,
}

impl FlagChecks {
    pub const fn new(
        check_z: Option<FlagCheck>,
        set_n: Option<FlagCheck>,
        half_carry: Option<FlagCheck>,
        carry: Option<FlagCheck>,
    ) -> Self {
        Self { check_z, set_n, half_carry, carry }
    }
}

pub enum FlagCheck {
    SetToValue(u8),
    Check,
    CheckOverflowAtBit(u8),
}

pub type InstructionResult<T> = Result<T, InstructionError>;

#[derive(Debug, Clone, Copy)]
pub enum InstructionError {
    InvalidOperation(u8),
    LdhLowValue(u16),
    MemoryAccessError(u16),
    OperandCannotBeSet,
}

impl From<MemoryAccessError> for InstructionError {
    fn from(value: MemoryAccessError) -> Self {
        match value {
            MemoryAccessError::NotAnOperation(_) => todo!(),
            MemoryAccessError::FailedToReadAddress => todo!(),
        }
    }
}
