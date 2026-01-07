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
    flags: FlagSettings,
}

impl Instruction {
    pub const fn new(operation: Operation, cycles: u8, bytes: u8, flags: FlagSettings) -> Self {
        Self { operation, cycles, bytes, flags }
    }

    pub fn get_operation(&self) -> &Operation {
        &self.operation
    }
    pub fn get_length(&self) -> u8 {
        self.bytes
    }
}

impl TryFrom<[u8; 3]> for Instruction {
    type Error = InstructionError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        todo!()
    }
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

pub enum OpCode {}

enum Operation {}

impl Operation {}

pub struct FlagSettings {}

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

    fn peek_stack(&self) -> u16 {
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
        todo!()
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
