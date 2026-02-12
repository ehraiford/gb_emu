use std::fmt::Display;

use crate::processor::{
    instruction_tables::{CBPREFIXED, UNPREFIXED},
    instructions::m_cycle_accuracy::{MicroOp, get_step_table_entry},
};

pub struct Instruction {
    pub op_code: OpCode,
    pub operands: &'static [Operand],
    pub cycles: u8,
    pub bytes: u16,
    pub steps: &'static [MicroOp],
}

impl Instruction {
    pub const fn new(op_code: OpCode, operands: &'static [Operand], cycles: u8, bytes: u16) -> Self {
        Self {
            op_code,
            operands,
            cycles,
            bytes,
            steps: get_step_table_entry(op_code, operands),
        }
    }

    pub const fn nop() -> &'static Self {
        &UNPREFIXED[0]
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
impl Display for &Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.op_code)?;

        for op in self.operands {
            write!(f, " {}", op)?;
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Adc = 0,
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
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OpCode::Adc => "ADC",
            OpCode::Add => "ADD",
            OpCode::And => "AND",
            OpCode::Bit => "BIT",
            OpCode::Call => "CALL",
            OpCode::Ccf => "CCF",
            OpCode::Cp => "CP",
            OpCode::Cpl => "CPL",
            OpCode::Daa => "DAA",
            OpCode::Dec => "DEC",
            OpCode::Di => "DI",
            OpCode::Ei => "EI",
            OpCode::Halt => "HALT",
            OpCode::Illegal => "ILLEGAL",
            OpCode::Inc => "INC",
            OpCode::Jp => "JP",
            OpCode::Jr => "JR",
            OpCode::Ld => "LD",
            OpCode::Ldh => "LDH",
            OpCode::Nop => "NOP",
            OpCode::Or => "OR",
            OpCode::Pop => "POP",
            OpCode::Prefix => "PREFIX",
            OpCode::Push => "PUSH",
            OpCode::Res => "RES",
            OpCode::Ret => "RET",
            OpCode::Reti => "RETI",
            OpCode::Rl => "RL",
            OpCode::Rla => "RLA",
            OpCode::Rlc => "RLC",
            OpCode::Rlca => "RLCA",
            OpCode::Rr => "RR",
            OpCode::Rra => "RRA",
            OpCode::Rrc => "RRC",
            OpCode::Rrca => "RRCA",
            OpCode::Rst => "RST",
            OpCode::Sbc => "SBC",
            OpCode::Scf => "SCF",
            OpCode::Set => "SET",
            OpCode::Sla => "SLA",
            OpCode::Sra => "SRA",
            OpCode::Srl => "SRL",
            OpCode::Stop => "STOP",
            OpCode::Sub => "SUB",
            OpCode::Swap => "SWAP",
            OpCode::Xor => "XOR",
        };

        f.write_str(str)
    }
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
    HLDPointer,
    HLIPointer,
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

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Operand::A => "A",
            Operand::A16 => "A16",
            Operand::A16Pointer => "[A16]",
            Operand::AF => "AF",
            Operand::B => "B",
            Operand::BC => "BC",
            Operand::BCPointer => "[BC]",
            Operand::C => "C",
            Operand::D => "D",
            Operand::DE => "DE",
            Operand::DEPointer => "[DE]",
            Operand::E => "E",
            Operand::E8 => "E8",
            Operand::FF00OffsetByA8 => "FF00+A8",
            Operand::FF00OffsetByC => "FF00+C",
            Operand::H => "H",
            Operand::HL => "HL",
            Operand::HLDPointer => "[HL-]",
            Operand::HLIPointer => "[HL+]",
            Operand::HLPointer => "[HL]",
            Operand::Immediate(imm) => &imm.to_string(),
            Operand::L => "L",
            Operand::N16 => "N16",
            Operand::N8 => "N8",
            Operand::Carry => "Carry",
            Operand::NotCarry => "NotCarry",
            Operand::NotZero => "NotZero",
            Operand::SP => "SP",
            Operand::Zero => "Zero",
        };

        f.write_str(str)
    }
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
            Operand::HLDPointer => Self::EightBitOperand,
            Operand::HLIPointer => Self::EightBitOperand,
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

#[derive(Debug, Clone, Copy)]
pub enum InstructionError {
    InvalidOperation(u8),
    InvalidOperand,
    LdhLowValue(u16),
    OperandCannotBeSet,
    MemoryAccessError,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    FF00OffsetByA8,
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
            Operand::FF00OffsetByA8 => Ok(Self::FF00OffsetByA8),
            Operand::FF00OffsetByC => Ok(Self::FF00OffsetByC),
            Operand::H => Ok(Self::H),
            Operand::N8 => Ok(Self::N8),
            Operand::Immediate(val) => Ok(Self::Immediate(val)),
            Operand::HLDPointer => Ok(Self::HLDPointer),
            Operand::HLIPointer => Ok(Self::HLIPointer),
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
    E8,
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
            Operand::E8 => Ok(Self::E8),
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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

impl From<Operand> for Condition {
    fn from(operand: Operand) -> Self {
        match operand {
            Operand::NotZero => Self::NotZero,
            Operand::Zero => Self::Zero,
            Operand::Carry => Self::Carry,
            Operand::NotCarry => Self::NotCarry,
            _ => unreachable!(),
        }
    }
}

pub mod m_cycle_accuracy {
    use crate::processor::instructions::{OpCode, Operand};

    pub enum MicroOp {
        Decode,
        ReadIntoOperand1,
        ReadIntoOperand0,
        Write,
        WriteBackOperand0,
        WriteBackOperand1,
        PopLsb,
        PopMsb,
        ReadE8Operand0,
        ReadE8Operand1,
        ReadSPPlusE8,
        ReadE8AndCheckCondition,
        WriteSPLow,
        WriteSPHigh,
        PushMsb,
        PushLsb,
        ReadIntoOperand0Lsb,
        ReadIntoOperand1Lsb,
        ReadIntoOperand1Msb,
        ReadIntoOperand1MsbAndCheckCondition,
        ReadIntoOperand0Msb,
        Wait,
        CheckCondition,
        CbPrefix,
        PopStackIntoLsbPc,
        PopStackIntoMsbPc,
        PushMsbPCToStackOperand0,
        PushMsbPCToStackOperand1,
        PushLsbPcToStackOperand0,
        PushLsbPcToStackOperand1,
        Illegal,
    }

    #[derive(PartialEq, Clone, Copy, Debug)]
    #[repr(u8)]
    enum OperandType {
        A = 0,
        R8,
        R16,
        Sp,
        Hl,
        Imm3Bit,
        Cond,
        N8,
        N16,
        N16Pointer,
        HLPointer,
        BCPointer,
        DEPointer,

        FF00PlusC,
        FF00PlusA8,
        HLDecrementPointer,
        HLIncrementPointer,
        E8,
    }

    impl OperandType {
        const fn derive_from_value(value: Operand) -> Self {
            match value {
                Operand::A => A,
                Operand::A16 => N16,
                Operand::A16Pointer => N16Pointer,
                Operand::BC | Operand::DE | Operand::AF => R16,
                Operand::C | Operand::D | Operand::B | Operand::E | Operand::L | Operand::H => R8,
                Operand::BCPointer => BCPointer,
                Operand::DEPointer => DEPointer,
                Operand::E8 => E8,
                Operand::FF00OffsetByA8 => FF00PlusA8,
                Operand::FF00OffsetByC => FF00PlusC,
                Operand::HL => Hl,
                Operand::HLDPointer => HLDecrementPointer,
                Operand::HLIPointer => HLIncrementPointer,
                Operand::HLPointer => HLPointer,
                Operand::Immediate(_) => Imm3Bit,
                Operand::N16 => N16,
                Operand::N8 => N8,
                Operand::Carry | Operand::NotCarry | Operand::NotZero | Operand::Zero => Cond,
                Operand::SP => Sp,
            }
        }
    }

    use MicroOp::*;
    use OpCode::*;
    use OperandType::*;

    pub struct StepTableEntry {
        op_code: OpCode,
        operands_types: &'static [OperandType],
        steps: &'static [MicroOp],
    }

    impl StepTableEntry {
        const fn new(op_code: OpCode, operands_types: &'static [OperandType], steps: &'static [MicroOp]) -> Self {
            Self { op_code, operands_types, steps }
        }
    }
    /// This table is derived from Gekkio's Game Boy: Complete Technical Reference,
    /// available at https://gekkio.fi/files/gb-docs/gbctr.pdf.
    /// It has been modified slightly to work a bit better for the state machine in my implementation of the CPU.
    pub const STEP_TABLE: &[StepTableEntry] = &[
        StepTableEntry::new(Ld, &[R8, R8], &[Decode]),
        StepTableEntry::new(Ld, &[R8, A], &[Decode]),
        StepTableEntry::new(Ld, &[A, R8], &[Decode]),
        StepTableEntry::new(Ld, &[A, A], &[Decode]),
        StepTableEntry::new(Ld, &[R8, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[R8, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[HLPointer, R8], &[Decode, Write]),
        StepTableEntry::new(Ld, &[HLPointer, A], &[Decode, Write]),
        StepTableEntry::new(Ld, &[HLPointer, N8], &[Decode, ReadIntoOperand1, Write]),
        StepTableEntry::new(Ld, &[A, BCPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[A, DEPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[BCPointer, A], &[Decode, Write]),
        StepTableEntry::new(Ld, &[DEPointer, A], &[Decode, Write]),
        StepTableEntry::new(
            Ld,
            &[A, N16Pointer],
            &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1Msb, ReadIntoOperand1],
        ),
        StepTableEntry::new(
            Ld,
            &[N16Pointer, A],
            &[Decode, ReadIntoOperand0Lsb, ReadIntoOperand0Msb, Write],
        ),
        StepTableEntry::new(Ldh, &[A, FF00PlusC], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ldh, &[FF00PlusC, A], &[Decode, Write]),
        StepTableEntry::new(Ldh, &[A, FF00PlusA8], &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1]),
        StepTableEntry::new(Ldh, &[FF00PlusA8, A], &[Decode, ReadIntoOperand0Lsb, Write]),
        StepTableEntry::new(Ld, &[A, HLDecrementPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[HLDecrementPointer, A], &[Decode, Write]),
        StepTableEntry::new(Ld, &[A, HLIncrementPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ld, &[HLIncrementPointer, A], &[Decode, Write]),
        StepTableEntry::new(Ld, &[R16, N16], &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1Msb]),
        StepTableEntry::new(Ld, &[Hl, N16], &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1Msb]),
        StepTableEntry::new(Ld, &[Sp, N16], &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1Msb]),
        StepTableEntry::new(
            Ld,
            &[N16Pointer, Sp],
            &[
                Decode,
                ReadIntoOperand0Lsb,
                ReadIntoOperand0Msb,
                WriteSPLow,
                WriteSPHigh,
            ],
        ),
        StepTableEntry::new(Ld, &[Sp, Hl], &[Decode, Wait]),
        StepTableEntry::new(Push, &[R16], &[Decode, Wait, PushMsb, PushLsb]),
        StepTableEntry::new(Push, &[Hl], &[Decode, Wait, PushMsb, PushLsb]),
        StepTableEntry::new(Pop, &[R16], &[Decode, PopLsb, PopMsb]),
        StepTableEntry::new(Pop, &[Hl], &[Decode, PopLsb, PopMsb]),
        StepTableEntry::new(Ld, &[Hl, Sp, E8], &[Decode, ReadSPPlusE8, Wait]),
        StepTableEntry::new(Add, &[A, R8], &[Decode]),
        StepTableEntry::new(Add, &[A, A], &[Decode]),
        StepTableEntry::new(Add, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Add, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Adc, &[A, R8], &[Decode]),
        StepTableEntry::new(Adc, &[A, A], &[Decode]),
        StepTableEntry::new(Adc, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Adc, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Sub, &[A, R8], &[Decode]),
        StepTableEntry::new(Sub, &[A, A], &[Decode]),
        StepTableEntry::new(Sub, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Sub, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Sbc, &[A, R8], &[Decode]),
        StepTableEntry::new(Sbc, &[A, A], &[Decode]),
        StepTableEntry::new(Sbc, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Sbc, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Cp, &[A, R8], &[Decode]),
        StepTableEntry::new(Cp, &[A, A], &[Decode]),
        StepTableEntry::new(Cp, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Cp, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Inc, &[R8], &[Decode]),
        StepTableEntry::new(Inc, &[A], &[Decode]),
        StepTableEntry::new(Inc, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Dec, &[R8], &[Decode]),
        StepTableEntry::new(Dec, &[A], &[Decode]),
        StepTableEntry::new(Dec, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(And, &[A, R8], &[Decode]),
        StepTableEntry::new(And, &[A, A], &[Decode]),
        StepTableEntry::new(And, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(And, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Or, &[A, R8], &[Decode]),
        StepTableEntry::new(Or, &[A, A], &[Decode]),
        StepTableEntry::new(Or, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Or, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Xor, &[A, R8], &[Decode]),
        StepTableEntry::new(Xor, &[A, A], &[Decode]),
        StepTableEntry::new(Xor, &[A, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Xor, &[A, N8], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Ccf, &[], &[Decode]),
        StepTableEntry::new(Scf, &[], &[Decode]),
        StepTableEntry::new(Daa, &[], &[Decode]),
        StepTableEntry::new(Cpl, &[], &[Decode]),
        StepTableEntry::new(Inc, &[R16], &[Decode, Wait]),
        StepTableEntry::new(Inc, &[Hl], &[Decode, Wait]),
        StepTableEntry::new(Inc, &[Sp], &[Decode, Wait]),
        StepTableEntry::new(Dec, &[R16], &[Decode, Wait]),
        StepTableEntry::new(Dec, &[Sp], &[Decode, Wait]),
        StepTableEntry::new(Dec, &[Hl], &[Decode, Wait]),
        StepTableEntry::new(Add, &[Hl, R16], &[Decode, Wait]),
        StepTableEntry::new(Add, &[Hl, Hl], &[Decode, Wait]),
        StepTableEntry::new(Add, &[Hl, Sp], &[Decode, Wait]),
        StepTableEntry::new(Add, &[Sp, E8], &[Decode, ReadE8Operand1, Wait, Wait]),
        StepTableEntry::new(Rlca, &[], &[Decode]),
        StepTableEntry::new(Rrca, &[], &[Decode]),
        StepTableEntry::new(Rla, &[], &[Decode]),
        StepTableEntry::new(Rra, &[], &[Decode]),
        StepTableEntry::new(Rlc, &[R8], &[Decode]),
        StepTableEntry::new(Rlc, &[A], &[Decode]),
        StepTableEntry::new(Rlc, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Rrc, &[R8], &[Decode]),
        StepTableEntry::new(Rrc, &[A], &[Decode]),
        StepTableEntry::new(Rrc, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Rl, &[R8], &[Decode]),
        StepTableEntry::new(Rl, &[A], &[Decode]),
        StepTableEntry::new(Rl, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Rr, &[R8], &[Decode]),
        StepTableEntry::new(Rr, &[A], &[Decode]),
        StepTableEntry::new(Rr, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Sla, &[R8], &[Decode]),
        StepTableEntry::new(Sla, &[A], &[Decode]),
        StepTableEntry::new(Sla, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Sra, &[R8], &[Decode]),
        StepTableEntry::new(Sra, &[A], &[Decode]),
        StepTableEntry::new(Sra, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Swap, &[R8], &[Decode]),
        StepTableEntry::new(Swap, &[A], &[Decode]),
        StepTableEntry::new(Swap, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Srl, &[R8], &[Decode]),
        StepTableEntry::new(Srl, &[A], &[Decode]),
        StepTableEntry::new(Srl, &[HLPointer], &[Decode, ReadIntoOperand0, WriteBackOperand0]),
        StepTableEntry::new(Bit, &[Imm3Bit, R8], &[Decode]),
        StepTableEntry::new(Bit, &[Imm3Bit, A], &[Decode]),
        StepTableEntry::new(Bit, &[Imm3Bit, HLPointer], &[Decode, ReadIntoOperand1]),
        StepTableEntry::new(Res, &[Imm3Bit, R8], &[Decode]),
        StepTableEntry::new(Res, &[Imm3Bit, A], &[Decode]),
        StepTableEntry::new(
            Res,
            &[Imm3Bit, HLPointer],
            &[Decode, ReadIntoOperand1, WriteBackOperand1],
        ),
        StepTableEntry::new(Set, &[Imm3Bit, R8], &[Decode]),
        StepTableEntry::new(Set, &[Imm3Bit, A], &[Decode]),
        StepTableEntry::new(
            Set,
            &[Imm3Bit, HLPointer],
            &[Decode, ReadIntoOperand1, WriteBackOperand1],
        ),
        StepTableEntry::new(Jp, &[N16], &[Decode, ReadIntoOperand0Lsb, ReadIntoOperand0Msb, Wait]),
        StepTableEntry::new(Jp, &[Hl], &[Decode]),
        StepTableEntry::new(
            Jp,
            &[Cond, N16],
            &[Decode, ReadIntoOperand1Lsb, ReadIntoOperand1MsbAndCheckCondition, Wait],
        ), // ENDS EARLY IF CONDITION NOT MET
        StepTableEntry::new(Jr, &[E8], &[Decode, ReadE8Operand0, Wait]),
        StepTableEntry::new(Jr, &[Cond, E8], &[Decode, ReadE8AndCheckCondition, Wait]), // ENDS EARLY IF CONDITION NOT MET
        StepTableEntry::new(
            Call,
            &[N16],
            &[
                Decode,
                ReadIntoOperand0Lsb,
                ReadIntoOperand0Msb,
                Wait,
                PushMsbPCToStackOperand0,
                PushLsbPcToStackOperand0,
            ],
        ),
        StepTableEntry::new(
            Call,
            &[Cond, N16],
            &[
                Decode,
                ReadIntoOperand1Lsb,
                ReadIntoOperand1MsbAndCheckCondition,
                Wait,
                PushMsbPCToStackOperand1,
                PushLsbPcToStackOperand1,
            ],
        ), // ENDS EARLY IF CONDITION NOT MET
        StepTableEntry::new(Ret, &[], &[Decode, PopStackIntoLsbPc, PopStackIntoMsbPc, Wait]),
        StepTableEntry::new(
            Ret,
            &[Cond],
            &[Decode, CheckCondition, PopStackIntoLsbPc, PopStackIntoMsbPc, Wait],
        ), // ENDS EARLY IF CONDITION NOT MET
        StepTableEntry::new(Reti, &[], &[Decode, PopStackIntoLsbPc, PopStackIntoMsbPc, Wait]),
        StepTableEntry::new(
            Rst,
            &[Imm3Bit],
            &[Decode, Wait, PushMsbPCToStackOperand0, PushLsbPcToStackOperand0],
        ),
        StepTableEntry::new(Halt, &[], &[Decode]),
        StepTableEntry::new(Stop, &[], &[Decode]),
        StepTableEntry::new(Di, &[], &[Decode]),
        StepTableEntry::new(Ei, &[], &[Decode]),
        StepTableEntry::new(Nop, &[], &[Decode]),
        StepTableEntry::new(Prefix, &[], &[CbPrefix]),
        StepTableEntry::new(OpCode::Illegal, &[], &[MicroOp::Illegal]),
    ];

    const fn is_a_match(opcode: OpCode, operands: &[Operand], step_table_entry: &StepTableEntry) -> bool {
        if opcode as u8 != step_table_entry.op_code as u8 {
            return false;
        }

        if operands.len() != step_table_entry.operands_types.len() {
            return false;
        }
        let mut i = 0;
        while i < operands.len() {
            if OperandType::derive_from_value(operands[i]) as u8 != step_table_entry.operands_types[i] as u8 {
                return false;
            }
            i += 1;
        }

        return true;
    }

    pub const fn get_step_table_entry(opcode: OpCode, operands: &[Operand]) -> &'static [MicroOp] {
        let mut i = 0;

        while i < STEP_TABLE.len() {
            if is_a_match(opcode, operands, &STEP_TABLE[i]) {
                return &STEP_TABLE[i].steps;
            } else {
                i += 1;
            }
        }

        panic!("No associated steps found.");
    }

    #[cfg(test)]
    mod test {
        use crate::processor::{
            instruction_tables::{CBPREFIXED, UNPREFIXED},
            instructions::{OpCode, m_cycle_accuracy::get_step_table_entry},
        };

        #[test]
        fn test_step_table_instr_lengths() {
            for i in (0..256).rev() {
                let instruction = &UNPREFIXED[i];
                if instruction.op_code == OpCode::Illegal || instruction.op_code == OpCode::Prefix {
                    continue;
                }
                // println!("Checking {}", instruction);
                let table_entry = get_step_table_entry(instruction.op_code, instruction.operands);
                assert_eq!(instruction.cycles, table_entry.len() as u8)
            }
            for i in (0..256).rev() {
                let instruction = &CBPREFIXED[i];
                let table_entry = get_step_table_entry(instruction.op_code, instruction.operands);
                assert_eq!(instruction.cycles - 1, table_entry.len() as u8)
            }
        }
    }
}
