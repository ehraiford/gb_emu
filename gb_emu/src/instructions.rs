use crate::{
    bus::MemoryAccessError,
    game_boy::Mode,
    instruction_tables::{CBPREFIXED, UNPREFIXED},
};

pub struct Instruction {
    pub op_code: OpCode,
    pub operands: &'static [Operand],
    pub cycles: u8,
    pub bytes: u16,
}

impl Instruction {
    pub const fn new(op_code: OpCode, operands: &'static [Operand], cycles: u8, bytes: u16) -> Self {
        Self { op_code, operands, cycles, bytes }
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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    FF00OffsetByA8,
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
    Carry,
    NotCarry,
    NotZero,
    SP,
    Zero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandType {
    EightBitOperand,
    SixteenBitOperand,
    SignedEightBit,
    Condition,
}

impl From<Operand> for OperandType {
    fn from(value: Operand) -> Self {
        match value {
            Operand::A => Self::EightBitOperand,
            Operand::A16Pointer => Self::EightBitOperand,
            Operand::B => Self::EightBitOperand,
            Operand::N8 => Self::EightBitOperand,
            Operand::BCPointer => Self::EightBitOperand,
            Operand::C => Self::EightBitOperand,
            Operand::D => Self::EightBitOperand,
            Operand::DEPointer => Self::EightBitOperand,
            Operand::E => Self::EightBitOperand,
            Operand::FF00OffsetByA8 => Self::EightBitOperand,
            Operand::FF00OffsetByC => Self::EightBitOperand,
            Operand::H => Self::EightBitOperand,
            Operand::HLD => Self::EightBitOperand,
            Operand::HLI => Self::EightBitOperand,
            Operand::HLPointer => Self::EightBitOperand,
            Operand::Immediate(_) => Self::EightBitOperand,
            Operand::L => Self::EightBitOperand,
            Operand::HL => Self::SixteenBitOperand,
            Operand::AF => Self::SixteenBitOperand,
            Operand::BC => Self::SixteenBitOperand,
            Operand::DE => Self::SixteenBitOperand,
            Operand::SP => Self::SixteenBitOperand,
            Operand::A16 => Self::SixteenBitOperand,
            Operand::N16 => Self::SixteenBitOperand,
            Operand::Carry => Self::Condition,
            Operand::NotCarry => Self::Condition,
            Operand::NotZero => Self::Condition,
            Operand::Zero => Self::Condition,
            Operand::E8 => Self::SignedEightBit,
        }
    }
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
    InvalidOperand,
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

#[derive(Copy, Clone, Debug)]
pub enum EightBitOperand {
    A,
    A16Pointer,
    B,
    HLPointer,
    HLIPointer,
    HLDPointer,
    BCPointer,
    L,
    C,
    D,
    DEPointer,
    E,
    FF00OffsetByA,
    FF00OffsetByC,
    H,
    N8,
    Immediate(u8),
}

impl TryFrom<Operand> for EightBitOperand {
    type Error = InstructionError;

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::A => Ok(Self::A),
            Operand::A16Pointer => Ok(Self::A16Pointer),
            Operand::B => Ok(Self::B),
            Operand::HLPointer => Ok(Self::HLPointer),
            Operand::BCPointer => Ok(Self::BCPointer),
            Operand::L => Ok(Self::L),
            Operand::C => Ok(Self::C),
            Operand::D => Ok(Self::D),
            Operand::DEPointer => Ok(Self::DEPointer),
            Operand::E => Ok(Self::E),
            Operand::FF00OffsetByA8 => Ok(Self::FF00OffsetByA),
            Operand::FF00OffsetByC => Ok(Self::FF00OffsetByC),
            Operand::H => Ok(Self::H),
            Operand::N8 => Ok(Self::N8),
            Operand::Immediate(val) => Ok(Self::Immediate(val)),
            Operand::HLD => Ok(Self::HLDPointer),
            Operand::HLI => Ok(Self::HLDPointer),
            _ => Err(InstructionError::InvalidOperand),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SixteenBitOperand {
    BC,
    DE,
    HL,
    AF,
    A16,
    N16,
    SP,
    Immediate(u16),
}

impl TryFrom<Operand> for SixteenBitOperand {
    type Error = InstructionError;

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::BC => Ok(Self::BC),
            Operand::DE => Ok(Self::DE),
            Operand::HL => Ok(Self::HL),
            Operand::AF => Ok(Self::AF),
            Operand::A16 => Ok(Self::A16),
            Operand::N16 => Ok(Self::N16),
            Operand::SP => Ok(Self::SP),
            _ => Err(InstructionError::InvalidOperand),
        }
    }
}
/// It feels silly to have this for just one type but it keeps format consistent.
/// Plus, Rust Enums are Zero-Cost Abstractions, right?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignedEightBitOperand {
    I8,
}

impl TryFrom<Operand> for SignedEightBitOperand {
    type Error = InstructionError;

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        if value == Operand::E8 {
            Ok(SignedEightBitOperand::I8)
        } else {
            Err(InstructionError::InvalidOperand)
        }
    }
}

pub enum InstructionOutcome {
    ExtraCycles(u16),
    Ok,
    ChangeGameBoyMode(Mode),
}
