use std::ops::{Shl, ShlAssign, Shr};

use crate::{
    bus::Bus,
    cpu::{Condition, Cpu, Flag, R8, R16, R16Mem, R16Stk},
    helper_functions::{concat_2_bytes, get_bitfield},
};

pub struct Instruction {
    operation: Operation,
    cycles: u8,
    bytes: u8,
    flags: FlagCondition,
}

impl Instruction {
    pub fn get_operation(&self) -> &Operation {
        &self.operation
    }
    pub fn get_length(&self) -> u8 {
        self.bytes
    }
}

/// Methods to derive an instruction from bytes
impl Instruction {
    /// Instructions that don't encode a register in their first byte
    pub fn registerless_ops(bytes: [u8; 3]) -> Option<InstructionResult<Instruction>> {
        match bytes[0] {
            0xCB => Some(Ok(Instruction::from_cb_prefixed(bytes[1]))),
            0x00 => Some(Ok(Instruction::nop())),
            0b0000_1000 => Some(Ok(Instruction::load(
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
                Operand::R16(R16::SP),
            ))),
            0b0001_1000 => Some(Ok(Instruction::jump_relative(Operand::N8(bytes[1])))),
            0b0001_0000 => Some(Ok(Instruction::stop())),
            0b0111_0110 => Some(Ok(Instruction::halt())),
            0b1100_1001 => Some(Ok(Instruction::ret())),
            0b1101_1001 => Some(Ok(Instruction::return_from_interrupt())),
            0b1100_0011 => Some(Ok(Instruction::jump(Operand::N16(concat_2_bytes(bytes[1], bytes[2]))))),
            0b1110_1001 => Some(Ok(Instruction::jump(Operand::R16(R16::HL)))),
            0b1100_1101 => Some(Ok(Instruction::call(Operand::N16(concat_2_bytes(bytes[1], bytes[2]))))),
            0b1111_0011 => Some(Ok(Instruction::disable_interrupts())),
            0b1111_1011 => Some(Ok(Instruction::enable_interrupts())),
            0b1110_0010 => Some(Ok(Instruction::load_high(Operand::N8(bytes[1]), Operand::R8(R8::A)))),
            0b1110_0000 => Some(Ok(Instruction::load_high(
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
                Operand::R8(R8::A),
            ))), // todo!("Double check this")
            0b1110_1010 => Some(Ok(Instruction::load(
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
                Operand::R8(R8::A),
            ))),
            0b1111_0010 => Some(Ok(Instruction::load_high(Operand::R8(R8::A), Operand::N8(bytes[1])))),
            0b1111_0000 => Some(Ok(Instruction::load_high(
                Operand::R8(R8::A),
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
            ))),
            0b1111_1010 => Some(Ok(Instruction::load(
                Operand::R8(R8::A),
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
            ))),
            0b1110_1000 => Some(Ok(Instruction::add(Operand::R16(R16::SP), Operand::N8(bytes[1])))),
            0b1111_1000 => Some(Ok(Instruction::load(Operand::R16(R16::HL), Operand::SpE8(bytes[1])))),
            0b1111_1001 => Some(Ok(Instruction::load(Operand::R16(R16::SP), Operand::R16(R16::HL)))),
            0b0010_0111 => Some(Ok(Instruction::decimal_adjust_accumulator())),
            0b0010_1111 => Some(Ok(Instruction::complement())),
            0b0011_0111 => Some(Ok(Instruction::set_carry_flag())),
            0b0011_1111 => Some(Ok(Instruction::complement_carry_flag())),
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                Some(Err(InstructionError::InvalidOperation(bytes[0])))
            },
            _ => None,
        }
    }

    /// Instructions that encode an r16, r16stk, or r16mem register in their first byte
    pub fn r16_ops(bytes: [u8; 3]) -> Option<InstructionResult<Instruction>> {
        match bytes[0] & 0b1100_1111 {
            0b0000_0001 => {
                let r16_operand = Operand::R16(R16::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::load(
                    r16_operand,
                    Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
                )))
            },
            0b0000_0010 => {
                let r16_mem_operand = Operand::R16Mem(R16Mem::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::load(r16_mem_operand, Operand::R8(R8::A))))
            },
            0b0000_1010 => {
                let r16_mem_operand = Operand::R16Mem(R16Mem::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::load(Operand::R8(R8::A), r16_mem_operand)))
            },
            0b0000_0011 => {
                let r16_operand = Operand::R16(R16::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::increment(r16_operand)))
            },
            0b0000_1011 => {
                let r16_operand = Operand::R16(R16::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::decrement(r16_operand)))
            },
            0b0000_1001 => {
                let r16_operand = Operand::R16(R16::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::add(Operand::R16(R16::HL), r16_operand)))
            },
            0b1100_0001 => {
                let r16_stk_operand = Operand::R16Stk(R16Stk::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::pop(r16_stk_operand)))
            },
            0b1100_0101 => {
                let r16_stk_operand = Operand::R16Stk(R16Stk::try_from(get_bitfield(bytes[0], 5, 4)).unwrap());
                Some(Ok(Instruction::push(r16_stk_operand)))
            },
            _ => None,
        }
    }

    fn from_cb_prefixed(value: u8) -> Self {
        let chunk0 = value >> 6; // top 2 bits decide if it's a shift operation or bit addressing
        let chunk1 = value >> 3 & 0b111; // decides either which shift operation or which bit index
        let r8_operand = Operand::R8(R8::try_from(value & 0b111).expect("Can't fail. It's properly masked.")); // decides which r8 is our operand

        match chunk0 {
            0b00 => match chunk1 {
                0b000 => Instruction::rotate_left(r8_operand),
                0b001 => Instruction::rotate_right(r8_operand),
                0b010 => Instruction::rotate_left_through_carry(r8_operand),
                0b011 => Instruction::rotate_right_through_carry(r8_operand),
                0b100 => Instruction::shift_left_arithmetic(r8_operand),
                0b101 => Instruction::shift_right_artithmetic(r8_operand),
                0b110 => Instruction::swap(r8_operand),
                0b111 => Instruction::shift_right_logical(r8_operand),
                _ => unreachable!("value has been masked. Cannot be greater than 8."),
            },
            0b01 => Instruction::test_bit(Operand::U3(chunk1), r8_operand),
            0b10 => Instruction::clear_bit(Operand::U3(chunk1), r8_operand),
            0b11 => Instruction::set_bit(Operand::U3(chunk1), r8_operand),
            _ => unreachable!("value has been masked. Cannot be greater than 3."),
        }
    }

    /// Instructions that encode one or two R8 registers in their first byte
    pub fn r8_ops(bytes: [u8; 3]) -> Option<InstructionResult<Instruction>> {
        let bitfield = get_bitfield(bytes[0], 5, 3);
        let r8_5_3 = Operand::R8(R8::try_from(bitfield).unwrap());
        match bytes[0] & 0b1100_0111 {
            0b0000_0100 => return Some(Ok(Instruction::increment(r8_5_3))),
            0b0000__0101 => return Some(Ok(Instruction::decrement(r8_5_3))),
            0b0000_0110 => return Some(Ok(Instruction::load(r8_5_3, Operand::N8(bytes[1])))),
            0b1100_0111 => return Some(Ok(Instruction::call_vector(Operand::N8(bitfield)))), // This one isn't encoding an R8 but it fits nicely here.
            _ => (),
        };

        let bitfield = get_bitfield(bytes[0], 2, 0);
        let r8_2_0 = Operand::R8(R8::try_from(bitfield).unwrap());
        match bytes[0] & 0b1111_1000 {
            0b1000_0000 => return Some(Ok(Instruction::add(Operand::R8(R8::A), r8_2_0))),
            0b1000_1000 => return Some(Ok(Instruction::add_with_carry(r8_2_0))),
            0b1001_0000 => return Some(Ok(Instruction::subtract(r8_2_0))),
            0b1001_1000 => return Some(Ok(Instruction::subtract_with_carry(r8_2_0))),
            0b1010_0000 => return Some(Ok(Instruction::and(r8_2_0))),
            0b1010_1000 => return Some(Ok(Instruction::xor(r8_2_0))),
            0b1011_0000 => return Some(Ok(Instruction::or(r8_2_0))),
            0b1011_1000 => return Some(Ok(Instruction::compare(r8_2_0))),
            _ => (),
        }

        if bytes[0] & 0b1100_0000 == 0b0100_0000 {
            return Some(Ok(Instruction::load(r8_5_3, r8_2_0)));
        }

        None
    }

    /// Instructions that encode a condition in their first byte
    fn condition_ops(bytes: [u8; 3]) -> Option<InstructionResult<Instruction>> {
        let condition = Operand::CC(Condition::try_from(get_bitfield(bytes[0], 4, 3)).unwrap());

        match bytes[0] & 0b1110_0111 {
            0b0010_0000 => Some(Ok(Instruction::jump_relative_conditional(
                condition,
                Operand::N8(bytes[1]),
            ))),
            0b1100_0000 => Some(Ok(Instruction::return_conditional(condition))),
            0b1100_0010 => Some(Ok(Instruction::jump_conditional(
                condition,
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
            ))),
            0b1100_0100 => Some(Ok(Instruction::call_conditional(
                condition,
                Operand::N16(concat_2_bytes(bytes[1], bytes[2])),
            ))),
            _ => None,
        }
    }
}

impl Instruction {
    fn load_high(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::LoadHigh(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn add_with_carry(operand: Operand) -> Self {
        let operation = Operation::AddWithCarry(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn add(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::Add(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn compare(operand: Operand) -> Self {
        let operation = Operation::Compare(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn decrement(operand: Operand) -> Self {
        let operation = Operation::Decrement(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn increment(operand: Operand) -> Self {
        let operation = Operation::Increment(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn subtract_with_carry(operand: Operand) -> Self {
        let operation = Operation::SubtractWithCarry(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn subtract(operand: Operand) -> Self {
        let operation = Operation::Subtract(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn and(operand: Operand) -> Self {
        let operation = Operation::And(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn or(operand: Operand) -> Self {
        let operation = Operation::Or(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn xor(operand: Operand) -> Self {
        let operation = Operation::Xor(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn rotate_left_through_carry(operand: Operand) -> Self {
        let operation = Operation::RotateLeftThroughCarry(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn rotate_left(operand: Operand) -> Self {
        let operation = Operation::RotateLeft(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn rotate_right_through_carry(operand: Operand) -> Self {
        let operation = Operation::RotateRightThroughCarry(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn rotate_right(operand: Operand) -> Self {
        let operation = Operation::RotateRight(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn shift_left_arithmetic(operand: Operand) -> Self {
        let operation = Operation::ShiftLeftArithmetic(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn shift_right_artithmetic(operand: Operand) -> Self {
        let operation = Operation::ShiftRightArtithmetic(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn shift_right_logical(operand: Operand) -> Self {
        let operation = Operation::ShiftRightLogical(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn swap(operand: Operand) -> Self {
        let operation = Operation::Swap(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn call(operand: Operand) -> Self {
        let operation = Operation::Call(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn jump(operand: Operand) -> Self {
        let operation = Operation::Jump(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn jump_conditional(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::JumpConditional(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn jump_relative(operand: Operand) -> Self {
        let operation = Operation::JumpRelative(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn return_conditional(operand: Operand) -> Self {
        let operation = Operation::ReturnConditional(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn call_vector(operand: Operand) -> Self {
        let operation = Operation::CallVector(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn pop(operand: Operand) -> Self {
        let operation = Operation::Pop(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn push(operand: Operand) -> Self {
        let operation = Operation::Push(operand);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn load(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::Load(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn test_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::TestBit(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn clear_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::ClearBit(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn set_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::SetBit(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn call_conditional(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::CallConditional(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn jump_relative_conditional(operand0: Operand, operand1: Operand) -> Self {
        let operation = Operation::JumpRelativeConditional(operand0, operand1);
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn complement() -> Self {
        let operation = Operation::Complement;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn ret() -> Self {
        let operation = Operation::Return;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn return_from_interrupt() -> Self {
        let operation = Operation::ReturnFromInterrupt;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn complement_carry_flag() -> Self {
        let operation = Operation::ComplementCarryFlag;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn set_carry_flag() -> Self {
        let operation = Operation::SetCarryFlag;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn disable_interrupts() -> Self {
        let operation = Operation::DisableInterrupts;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn enable_interrupts() -> Self {
        let operation = Operation::EnableInterrupts;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn halt() -> Self {
        let operation = Operation::Halt;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn decimal_adjust_accumulator() -> Self {
        let operation = Operation::DecimalAdjustAccumulator;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn nop() -> Self {
        let operation = Operation::Nop;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
    fn stop() -> Self {
        let operation = Operation::Stop;
        let (cycles, bytes, flags) = operation.get_cycles_bytes_flags();
        Self { operation, cycles, bytes, flags }
    }
}

impl TryFrom<[u8; 3]> for Instruction {
    type Error = InstructionError;

    fn try_from(bytes: [u8; 3]) -> Result<Self, Self::Error> {
        if let Some(instruction) = Instruction::registerless_ops(bytes) {
            instruction
        } else if let Some(instruction) = Instruction::r16_ops(bytes) {
            instruction
        } else if let Some(instruction) = Instruction::r8_ops(bytes) {
            instruction
        } else if let Some(instruction) = Instruction::condition_ops(bytes) {
            instruction
        } else {
            unreachable!(
                "All variations should be covered above. Is 0x{:02x} not covered?",
                bytes[0]
            )
        }
    }
}
pub enum FlagCondition {
    NoneAffected,
    Check {
        check_z: bool,
        n_val: u8,
        h_bit: u8,
        c_bit: u8,
    },
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Operand {
    R8(R8),
    HlPointer,
    MemPointer(u16),
    R16(R16),
    R16Stk(R16Stk),
    R16Mem(R16Mem),
    N8(u8),
    N16(u16),
    E8(i8),
    SpE8(u8),
    U3(u8),
    CC(Condition),
}

pub enum Operation {
    Load(Operand, Operand),
    LoadHigh(Operand, Operand),
    AddWithCarry(Operand),
    Add(Operand, Operand),
    Compare(Operand),
    Decrement(Operand),
    Increment(Operand),
    SubtractWithCarry(Operand),
    Subtract(Operand),
    And(Operand),
    Complement,
    Or(Operand),
    Xor(Operand),
    TestBit(Operand, Operand),
    ClearBit(Operand, Operand),
    SetBit(Operand, Operand),
    RotateLeftThroughCarry(Operand),
    RotateLeft(Operand),
    RotateRightThroughCarry(Operand),
    RotateRight(Operand),
    ShiftLeftArithmetic(Operand),
    ShiftRightArtithmetic(Operand),
    ShiftRightLogical(Operand),
    Swap(Operand),
    Call(Operand),
    CallConditional(Operand, Operand),
    Jump(Operand),
    JumpConditional(Operand, Operand),
    JumpRelative(Operand),
    JumpRelativeConditional(Operand, Operand),
    Return,
    ReturnConditional(Operand),
    ReturnFromInterrupt,
    CallVector(Operand),
    ComplementCarryFlag,
    SetCarryFlag,
    Pop(Operand),
    Push(Operand),
    DisableInterrupts,
    EnableInterrupts,
    Halt,
    DecimalAdjustAccumulator,
    Nop,
    Stop,
}

impl Operation {
    /// todo!("See how much making this const actually helps")
    pub const fn get_cycles_bytes_flags(&self) -> (u8, u8, FlagCondition) {
        match self {
            Operation::Load(operand0, operand1) => {
                match (
                    OperandType::from_operand(*operand0),
                    OperandType::from_operand(*operand1),
                ) {
                    (OperandType::R8, OperandType::R8) => (1, 1, FlagCondition::NoneAffected),
                    (OperandType::R8, OperandType::N8) => (2, 2, FlagCondition::NoneAffected),
                    (OperandType::R16, OperandType::N16) => (3, 3, FlagCondition::NoneAffected),
                    (OperandType::HlPointer, OperandType::R8) => (2, 1, FlagCondition::NoneAffected),
                    (OperandType::HlPointer, OperandType::N8) => (3, 2, FlagCondition::NoneAffected),
                    (OperandType::R8, OperandType::HlPointer) => (2, 1, FlagCondition::NoneAffected),
                    (OperandType::R16, OperandType::R8) => (2, 1, FlagCondition::NoneAffected),
                    (OperandType::MemPointer, OperandType::R8) => (4, 3, FlagCondition::NoneAffected),
                    (OperandType::R16Mem, OperandType::R8) => (2, 1, FlagCondition::NoneAffected),
                    (OperandType::R8, OperandType::R16Mem) => (2, 1, FlagCondition::NoneAffected),
                    (OperandType::MemPointer, OperandType::R16) => (5, 3, FlagCondition::NoneAffected),
                    (OperandType::R16, OperandType::SpE8) => (
                        3,
                        2,
                        FlagCondition::Check { check_z: false, n_val: 0, h_bit: 3, c_bit: 7 },
                    ),
                    (OperandType::R16, OperandType::R16) => (2, 1, FlagCondition::NoneAffected),
                    _ => unreachable!(),
                }
            },
            Operation::LoadHigh(operand0, operand1) => match (
                OperandType::from_operand(*operand0),
                OperandType::from_operand(*operand1),
            ) {
                (OperandType::MemPointer, OperandType::R8) => (3, 2, FlagCondition::NoneAffected),
                (OperandType::N8, OperandType::R8) => (2, 1, FlagCondition::NoneAffected),
                (OperandType::R8, OperandType::MemPointer) => (3, 2, FlagCondition::NoneAffected),
                (OperandType::R8, OperandType::N8) => (2, 1, FlagCondition::NoneAffected),
                _ => unreachable!(),
            },
            Operation::AddWithCarry(operand) => match OperandType::from_operand(*operand) {
                OperandType::R8 => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                OperandType::HlPointer => (
                    2,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                OperandType::N8 => (
                    2,
                    2,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                _ => unreachable!(),
            },
            Operation::Add(operand0, operand1) => match (
                OperandType::from_operand(*operand0),
                OperandType::from_operand((*operand1)),
            ) {
                (OperandType::R8, OperandType::R8) => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                (OperandType::R8, OperandType::HlPointer) => (
                    2,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                (OperandType::R8, OperandType::N8) => (
                    2,
                    2,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                (OperandType::R16, OperandType::E8) => (
                    4,
                    2,
                    FlagCondition::Check { check_z: false, n_val: 0, h_bit: 3, c_bit: 7 },
                ),
                _ => unreachable!(),
            },
            Operation::Compare(operand) => match OperandType::from_operand(*operand) {
                OperandType::R8 => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                OperandType::HlPointer => (
                    2,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                OperandType::N8 => (
                    2,
                    2,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                _ => unreachable!(),
            },
            Operation::Decrement(operand) => match OperandType::from_operand(*operand) {
                OperandType::R8 => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 0 },
                ),
                OperandType::HlPointer => (
                    3,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 0 },
                ),
                OperandType::R16 => (2, 1, FlagCondition::NoneAffected),
                _ => unreachable!(),
            },
            Operation::Increment(operand) => match OperandType::from_operand(*operand) {
                OperandType::R8 => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 0 },
                ),
                OperandType::HlPointer => (
                    3,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 0 },
                ),
                OperandType::R16 => (
                    2,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 0, h_bit: 3, c_bit: 0 },
                ),
                _ => unreachable!(),
            },
            Operation::SubtractWithCarry(operand) => match OperandType::from_operand(*operand) {
                OperandType::R8 => (
                    1,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                OperandType::HlPointer => (
                    2,
                    1,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                OperandType::N8 => (
                    2,
                    2,
                    FlagCondition::Check { check_z: true, n_val: 1, h_bit: 4, c_bit: 8 },
                ),
                _ => unreachable!(),
            },
            Operation::Subtract(operand) => match OperandType::from_operand(*operand) {
                
                _ => unreachable!(),
            },
            Operation::And(operand) => todo!(),
            Operation::Or(operand) => todo!(),
            Operation::Xor(operand) => todo!(),
            Operation::TestBit(operand0, operand1) => todo!(),
            Operation::ClearBit(operand0, operand1) => todo!(),
            Operation::SetBit(operand0, operand1) => todo!(),
            Operation::RotateLeftThroughCarry(operand) => todo!(),
            Operation::RotateLeft(operand) => todo!(),
            Operation::RotateRightThroughCarry(operand) => todo!(),
            Operation::RotateRight(operand) => todo!(),
            Operation::ShiftLeftArithmetic(operand) => todo!(),
            Operation::ShiftRightArtithmetic(operand) => todo!(),
            Operation::ShiftRightLogical(operand) => todo!(),
            Operation::Swap(operand) => todo!(),
            Operation::Call(operand) => todo!(),
            Operation::CallConditional(operand0, operand1) => todo!(),
            Operation::Jump(operand) => todo!(),
            Operation::JumpConditional(operand0, operand1) => todo!(),
            Operation::JumpRelative(operand) => todo!(),
            Operation::JumpRelativeConditional(operand0, operand1) => todo!(),
            Operation::ReturnConditional(operand) => todo!(),
            Operation::CallVector(operand) => todo!(),
            Operation::Pop(operand) => todo!(),
            Operation::Push(operand) => todo!(),
            Operation::Complement => todo!(),
            Operation::Return => todo!(),
            Operation::ReturnFromInterrupt => todo!(),
            Operation::ComplementCarryFlag => todo!(),
            Operation::SetCarryFlag => todo!(),
            Operation::DisableInterrupts => todo!(),
            Operation::EnableInterrupts => todo!(),
            Operation::Halt => todo!(),
            Operation::DecimalAdjustAccumulator => todo!(),
            Operation::Nop => todo!(),
            Operation::Stop => todo!(),
        }
    }
    pub fn get_flags(&self) -> FlagCondition {
        todo!()
    }
}

pub struct OperationContext<'a, 'b> {
    cpu: &'a mut Cpu,
    bus: &'b mut Bus,
}

impl<'a, 'b> OperationContext<'a, 'b> {
    pub fn new(cpu: &'a mut Cpu, bus: &'b mut Bus) -> Self {
        Self { cpu, bus }
    }

    fn push_to_stack(&mut self, value: u16) {
        todo!();
    }

    fn pop_from_stack(&mut self) -> u16 {
        todo!()
    }

    fn peak_stack(&self) -> u16 {
        todo!()
    }

    fn check_condition(&self, cond: &Condition) -> bool {
        self.cpu.check_condition(cond)
    }

    fn get_a(&self) -> u8 {
        self.cpu.get_a()
    }

    fn set_a(&mut self, value: u8) {
        self.cpu.set_a(value)
    }

    fn get_operand(&mut self, operand: &Operand) -> u16 {
        self.cpu.get_operand(operand, self.bus)
    }
    fn set_operand(&mut self, operand: &Operand, value: u16) {
        self.cpu.set_operand(operand, value, self.bus);
    }

    pub fn perform_instruction(&mut self, instruction: &Instruction) {
        let result = match instruction.get_operation() {
            Operation::Load(operand0, operand1) => self.load(operand0, operand1),
            Operation::LoadHigh(operand0, operand1) => self.load_high(operand0, operand1),
            Operation::AddWithCarry(operand) => self.add_with_carry(operand),
            Operation::Add(operand0, operand1) => self.add(operand0, operand1),
            Operation::Compare(operand) => self.compare(operand),
            Operation::Decrement(operand) => self.decrement(operand),
            Operation::Increment(operand) => self.increment(operand),
            Operation::SubtractWithCarry(operand) => self.subtract_with_carry(operand),
            Operation::Subtract(operand) => self.subtract(operand),
            Operation::And(operand) => self.and(operand),
            Operation::Complement => self.complement(),
            Operation::Or(operand) => self.or(operand),
            Operation::Xor(operand) => self.xor(operand),
            Operation::TestBit(operand0, operand1) => self.test_bit(operand0, operand1),
            Operation::ClearBit(operand0, operand1) => self.clear_bit(operand0, operand1),
            Operation::SetBit(operand0, operand1) => self.set_bit(operand0, operand1),
            Operation::RotateLeftThroughCarry(operand) => self.rotate_left_through_carry(operand),
            Operation::RotateLeft(operand) => self.rotate_left(operand),
            Operation::RotateRightThroughCarry(operand) => self.rotate_right_through_carry(operand),
            Operation::RotateRight(operand) => self.rotate_right(operand),
            Operation::ShiftLeftArithmetic(operand) => self.shift_left_arithmetic(operand),
            Operation::ShiftRightArtithmetic(operand) => self.shift_right_artithmetic(operand),
            Operation::ShiftRightLogical(operand) => self.shift_right_logical(operand),
            Operation::Swap(operand) => self.swap(operand),
            Operation::Call(operand) => self.call(operand),
            Operation::CallConditional(operand0, operand1) => self.call_conditional(operand0, operand1),
            Operation::Jump(operand) => self.jump(operand),
            Operation::JumpConditional(operand0, operand1) => self.jump_conditional(operand0, operand1),
            Operation::JumpRelative(operand) => self.jump_relative(operand),
            Operation::JumpRelativeConditional(operand0, operand1) => {
                self.jump_relative_conditional(operand0, operand1)
            },
            Operation::Return => self.ret(),
            Operation::ReturnConditional(operand) => self.return_conditional(operand),
            Operation::ReturnFromInterrupt => self.return_from_interrupt(),
            Operation::CallVector(operand) => self.call_vector(operand),
            Operation::ComplementCarryFlag => self.complement_carry_flag(),
            Operation::SetCarryFlag => self.set_carry_flag(),
            Operation::Pop(operand) => self.pop(operand),
            Operation::Push(operand) => self.push(operand),
            Operation::DisableInterrupts => self.disable_interrupts(),
            Operation::EnableInterrupts => self.enable_interrupts(),
            Operation::Halt => self.halt(),
            Operation::DecimalAdjustAccumulator => self.decimal_adjust_accumulator(),
            Operation::Nop => self.nop(),
            Operation::Stop => self.stop(),
        };
    }

    fn load(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let value = self.get_operand(operand1);
        self.set_operand(operand0, value);

        Ok(value)
    }

    fn load_high(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let value = self.get_operand(operand1);

        if value < 0xFF00 {
            return Err(InstructionError::LdhLowValue(value));
        }
        self.set_operand(operand0, value);

        Ok(value)
    }

    fn add_with_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let carry = self.cpu.get_flag(Flag::Carry) as u16;
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = operand_value + a + carry;
        self.set_a(result as u8);

        Ok(result)
    }

    fn add(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let operand0_val = self.get_operand(operand0);
        let operand1_val = self.get_operand(operand1);

        let result = operand0_val + operand1_val;
        self.set_operand(operand0, result);

        Ok(result)
    }

    fn compare(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a.wrapping_sub(operand_value);

        Ok(result)
    }

    fn decrement(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value - 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn increment(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value + 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn subtract_with_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;
        let carry = self.cpu.get_flag(Flag::Carry) as u16;

        let result = a - carry - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn subtract(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn and(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a & operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn complement(&mut self) -> InstructionResult<u16> {
        let a = self.get_a();
        self.set_a(!a);

        Ok(!a as u16)
    }

    fn or(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a | operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn xor(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a ^ operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn test_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0);
        let test_value = self.get_operand(operand1);

        let result = (test_value >> index) & 0b1;

        Ok(result)
    }

    fn clear_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0);
        let mask = !(0b1 << index);
        let operand_value = self.get_operand(operand1);

        let result = operand_value & mask;
        self.set_operand(operand1, result);

        Ok(result)
    }

    fn set_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0);
        let mask = (0b1 << index);
        let operand_value = self.get_operand(operand1);

        let result = operand_value | mask;
        self.set_operand(operand1, result);

        Ok(result)
    }

    fn rotate_left_through_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let mut operand_value = self.get_operand(operand);
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;

        operand_value <<= 1;
        operand_value |= carry;
        carry = operand_value >> 8;

        self.set_operand(operand, operand_value);

        Ok(carry)
    }

    fn rotate_left(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let result = operand_value.wrapping_shl(1);
        let carry = result & 0b1;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn rotate_right_through_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let mut operand_value = self.get_operand(operand);
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;

        operand_value |= carry << 8;
        carry = operand_value & 0b1;
        operand_value >>= 1;

        self.set_operand(operand, operand_value);

        Ok(carry)
    }

    fn rotate_right(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let carry = operand_value & 0b1;
        let result = operand_value.wrapping_shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_left_arithmetic(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let result = operand_value.shl(1);
        let carry = result & 0b1u8;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_artithmetic(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand) as i8;

        let carry = operand_value & 0b1;
        let result = operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_logical(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let carry = operand_value & 0b1;
        let result = operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn swap(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value.wrapping_shl(4);

        self.set_operand(operand, result);

        Ok(result)
    }

    fn call(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let next_instruction_address = self.cpu.get_pc() + 3;
        let call_address = self.get_operand(operand);

        self.push_to_stack(next_instruction_address);
        self.cpu.set_pc(call_address);

        Ok(next_instruction_address)
    }

    fn call_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let Operand::CC(condition) = operand0 else {
            return Err(InstructionError::InvalidOperandType {
                expected: OperandType::CC,
                received: (*operand0).into(),
            });
        };

        if self.check_condition(condition) {
            self.call(operand1)
        } else {
            todo!("Adjust number of cycles taken")
        }
    }

    fn jump(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let address = self.get_operand(operand);

        self.cpu.set_pc(address);

        Ok(address)
    }

    fn jump_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let Operand::CC(condition) = operand0 else {
            return Err(InstructionError::InvalidOperandType {
                expected: OperandType::CC,
                received: (*operand0).into(),
            });
        };

        if self.check_condition(condition) {
            self.jump(operand1)
        } else {
            Ok(self.cpu.get_pc())
        }
    }

    fn jump_relative(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);
        let pc = self.cpu.get_pc();

        let result = operand_value + pc;
        self.cpu.set_pc(result);

        Ok(result)
    }

    fn jump_relative_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let Operand::CC(condition) = operand0 else {
            return Err(InstructionError::InvalidOperandType {
                expected: OperandType::CC,
                received: (*operand0).into(),
            });
        };

        if self.check_condition(condition) {
            self.jump_relative(operand1)
        } else {
            Ok(self.cpu.get_pc())
        }
    }

    fn return_conditional(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let Operand::CC(condition) = operand else {
            return Err(InstructionError::InvalidOperandType {
                expected: OperandType::CC,
                received: (*operand).into(),
            });
        };

        if self.check_condition(condition) {
            self.ret()
        } else {
            todo!("Change cycles")
        }
    }

    fn call_vector(&mut self, operand: &Operand) -> InstructionResult<u16> {
        self.call(operand)
    }

    fn ret(&mut self) -> InstructionResult<u16> {
        let new_pc = self.pop_from_stack();
        self.cpu.set_pc(new_pc);

        Ok(new_pc)
    }

    fn return_from_interrupt(&mut self) -> InstructionResult<u16> {
        self.enable_interrupts()?;
        self.ret()
    }

    fn complement_carry_flag(&mut self) -> InstructionResult<u16> {
        let carry = self.cpu.get_flag(Flag::Carry);
        self.cpu.set_flag(Flag::Carry, !(carry != 0));

        Ok(0)
    }

    fn set_carry_flag(&mut self) -> InstructionResult<u16> {
        self.cpu.set_flag(Flag::Carry, true);

        Ok(0)
    }

    fn pop(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let stack_value = self.pop_from_stack();

        self.set_operand(operand, stack_value);

        Ok(stack_value)
    }

    fn push(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand);

        self.push_to_stack(operand_value);

        Ok(operand_value)
    }

    fn disable_interrupts(&mut self) -> InstructionResult<u16> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, false);

        Ok(0)
    }

    fn enable_interrupts(&mut self) -> InstructionResult<u16> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, true);

        Ok(0)
    }

    fn halt(&mut self) -> InstructionResult<u16> {
        todo!()
    }

    fn decimal_adjust_accumulator(&mut self) -> InstructionResult<u16> {
        let mut adjustment = 0;
        match self.cpu.get_flag(Flag::Zero) {
            1 => {
                if self.cpu.get_flag(Flag::HalfCarry) == 1 {
                    adjustment += 0x6;
                }
                if self.cpu.get_flag(Flag::Carry) == 1 {
                    adjustment += 0x60;
                }
                self.subtract(&Operand::N8(adjustment))
            },
            _ => {
                let a = self.get_a();
                if self.cpu.get_flag(Flag::HalfCarry) == 1 || a & 0xF > 0x9 {
                    adjustment += 0x6;
                }
                if self.cpu.get_flag(Flag::Carry) == 1 || a > 0x99 {
                    adjustment += 0x60;
                }
                self.add(&Operand::R8(R8::A), &Operand::N8(adjustment))
            },
        }
    }

    fn nop(&mut self) -> InstructionResult<u16> {
        Ok(0)
    }

    fn stop(&mut self) -> InstructionResult<u16> {
        todo!()
    }
}

type InstructionResult<T> = Result<T, InstructionError>;

pub enum InstructionError {
    InvalidOperandType {
        expected: OperandType,
        received: OperandType,
    },
    InvalidOperation(u8),
    LdhLowValue(u16),
}
pub enum OperandType {
    R8,
    HlPointer, // Easier to keep this as a separate case than as an R8 variant
    MemPointer,
    R16,
    R16Stk,
    R16Mem,
    N8,
    N16,
    U3,
    CC,
    E8,
    SpE8,
}

impl OperandType {
    const fn from_operand(value: Operand) -> Self {
        match value {
            Operand::R8(_) => Self::R8,
            Operand::R16(_) => Self::R16,
            Operand::R16Stk(_) => Self::R16Stk,
            Operand::R16Mem(_) => Self::R16Mem,
            Operand::N8(_) => Self::N8,
            Operand::N16(_) => Self::N16,
            Operand::U3(_) => Self::U3,
            Operand::CC(_) => Self::CC,
            Operand::SpE8(_) => Self::SpE8,
            Operand::HlPointer => Self::HlPointer,
            Operand::MemPointer(_) => Self::MemPointer,
            Operand::E8(_) => Self::E8,
        }
    }
}

impl From<Operand> for OperandType {
    fn from(value: Operand) -> Self {
        Self::from_operand(value)
    }
}

#[cfg(test)]
mod test {
    use crate::instructions::Instruction;

    const INVALID_BYTES: &[u8] = &[0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD];

    #[test]
    /// Just tests that every possible value for an instruction is covered by our translations.
    /// It doesn't check that the translations are valid.
    fn test_instruction_coverage() {
        let mut bytes = [0, 0, 0];
        for i in 0..=255 {
            bytes[0] = i;
            // our try from will panic if there's a None at the end so we don't need to actually check it.
            // Just calling it for every byte will work.
            let instruction_result = Instruction::try_from(bytes);

            // the try_from should only fail on the invalid opcodes
            if instruction_result.is_err() {
                assert!(INVALID_BYTES.contains(&bytes[0]));
            }
        }
    }
}
