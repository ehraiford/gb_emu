use std::ops::{Shl, ShlAssign, Shr};

use crate::{bus::Bus, cpu::{Condition, Cpu, Flag, R8, R16, R16Mem, R16Stk}};


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

    pub fn from_cb_prefixed(value: u8) -> Self {
        let chunk0 = value >> 6; // top 2 bits decide if it's a shift operation or bit addressing
        let chunk1 = value >> 3 & 0b111; // decides either which shift operation or which bit index
        let r8_operand = Operand::R8(R8::try_from(value & 0b111).expect("Can't fail. It's properly masked.")); // decides which r8 is our operand

        match chunk0 {
            0b00 => {
                match chunk1 {
                    0b000 => Instruction::rotate_left(r8_operand),
                    0b001 => Instruction::rotate_right(r8_operand),
                    0b010 => Instruction::rotate_left_through_carry(r8_operand),
                    0b011 => Instruction::rotate_right_through_carry(r8_operand),
                    0b100 => Instruction::shift_left_arithmetic(r8_operand),
                    0b101 => Instruction::shift_right_artithmetic(r8_operand),
                    0b110 => Instruction::swap(r8_operand),
                    0b111 => Instruction::shift_right_logical(r8_operand),
                    _ => unreachable!("value has been masked. Cannot be greater than 8.")

                }
            },
            0b01 => Instruction::test_bit(Operand::U3(chunk1), r8_operand),
            0b10 => Instruction::clear_bit(Operand::U3(chunk1), r8_operand),
            0b11 => Instruction::set_bit(Operand::U3(chunk1), r8_operand),
            _ => unreachable!("value has been masked. Cannot be greater than 3.")
        }
    }
}

impl Instruction {
    fn load_high(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
        let flags = operation.get_flags();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,            
        }
    }
    fn add_with_carry(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn add(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn compare(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn decrement(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn increment(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn subtract_with_carry(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn subtract(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn and(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn or(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn xor(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn rotate_left_through_carry(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self 
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,     
        }
    fn rotate_left(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn rotate_right_through_carry(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn rotate_right(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn shift_left_arithmetic(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn shift_right_artithmetic(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn shift_right_logical(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn swap(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn call(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn jump(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn jump_relative(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn return_conditional(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn call_vector(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn pop(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn push(operand: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn load(operand0: Operand,operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn test_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn clear_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn set_bit(operand0: Operand, operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn call_conditional(operand0: Operand, operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn jump_relative_conditional(operand0: Operand, operand1: Operand) -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn complement() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn ret() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn return_from_interrupt() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn complement_carry_flag() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn set_carry_flag() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn disable_interrupts() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn enable_interrupts() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn halt() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn decimal_adjust_accumulator() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn nop() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
    fn stop() -> Self {
        let operation = todo!();
        let cycles = operation.get_cycles();
                Self {
                    operation,
                    cycles,
                    bytes: todo!(),
                    flags,    
        }
    }
}

impl TryFrom<[u8;3]> for Instruction {
    type Error = InstructionError;

    fn try_from(bytes: [u8;3]) -> Result<Self, Self::Error> {
        match bytes[0] {
            0xCB => Ok(Instruction::from_cb_prefixed(bytes[1])),
            0x00 => Ok(Instruction::nop()),
            0b0001_1000 => Ok(Instruction::jump_relative(Operand::N8(bytes[1]))),
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => Err(InstructionError::InvalidOperation(vec![bytes[0]]))
        }
    }
}
pub enum FlagCondition {

}

#[derive(Clone, Copy)]
pub enum Operand {
    R8(R8),
    R16(R16),
    R16Stk(R16Stk),
    R16Mem(R16Mem),
    N8(u8),
    N16(u16),
    U3(u8),
    CC(Condition)
}

pub enum Operation {
    Load(Operand,Operand),
    LoadHigh(Operand),
    AddWithCarry(Operand),
    Add(Operand),
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
}




pub struct OperationContext<'a, 'b> {
    cpu: &'a mut Cpu,
    bus: &'b mut Bus,
}

impl<'a, 'b> OperationContext<'a, 'b> {
    pub fn new(cpu: &'a mut Cpu,bus: &'b mut Bus) -> Self {
        Self {
            cpu,
            bus,
        }
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
    fn set_operand(&mut self, operand: &Operand, value: u16)  {
        self.cpu.set_operand(operand, value, self.bus);
    }

    pub fn perform_instruction(&mut self, instruction: &Instruction) {
        let result = match instruction.get_operation() {
            Operation::Load(operand0, operand1) => self.load(operand0, operand1),
            Operation::LoadHigh(operand) => self.load_high(operand),
            Operation::AddWithCarry(operand) => self.add_with_carry(operand),
            Operation::Add(operand) => self.add(operand),
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
            Operation::JumpRelative(operand) => self.jump_relative(operand),
            Operation::JumpRelativeConditional(operand0, operand1) => self.jump_relative_conditional(operand0, operand1),
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

    fn load(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let value = self.get_operand(operand1);
        self.set_operand(operand0, value);
        
        Ok(value)
    }

    fn load_high(&mut self, operand: &Operand) -> OperationResult<u16> {
        let value = self.get_a() as u16;
        self.set_operand(operand, value);

        Ok(value)
    }
    
    fn add_with_carry(&mut self, operand: &Operand) -> OperationResult<u16> {
        let carry = self.cpu.get_flag(Flag::Carry) as u16;
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;
        
        let result = operand_value + a + carry;
        self.set_a(result as u8);

        Ok(result)
    }

    fn add(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = operand_value + a;
        self.set_a(result as u8);

        Ok(result)
    }

    fn compare(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a.wrapping_sub(operand_value);

        Ok(result)        
    }

    fn decrement(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value - 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn increment(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value + 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn subtract_with_carry(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;
        let carry = self.cpu.get_flag(Flag::Carry) as u16;

        let result = a - carry - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn subtract(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn and(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a & operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn complement(&mut self) -> OperationResult<u16> {
        let a = self.get_a();
        self.set_a(!a);

        Ok(!a as u16)
    }

    fn or(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a | operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn xor(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let a = self.get_a() as u16;

        let result = a ^ operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn test_bit(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let index = self.get_operand(operand0);
        let test_value = self.get_operand(operand1);
        
        let result = (test_value >> index) & 0b1;

        Ok(result)
    }

    fn clear_bit(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let index = self.get_operand(operand0);
        let mask = !(0b1 << index); 
        let operand_value = self.get_operand(operand1);

        let result = operand_value & mask;
        self.set_operand(operand1, result);

        Ok(result)

    }

    fn set_bit(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let index = self.get_operand(operand0);
        let mask = (0b1 << index); 
        let operand_value = self.get_operand(operand1);

        let result = operand_value | mask;
        self.set_operand(operand1, result);

        Ok(result)    }

    fn rotate_left_through_carry(&mut self, operand: &Operand) -> OperationResult<u16> {
        let mut operand_value = self.get_operand(operand);
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;

        operand_value <<= 1;
        operand_value |= carry;
        carry = operand_value >> 8;

        self.set_operand(operand, operand_value);

        Ok(carry)
    }

    fn rotate_left(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let result = operand_value.wrapping_shl(1);
        let carry = result & 0b1;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn rotate_right_through_carry(&mut self, operand: &Operand) -> OperationResult<u16> {
        let mut operand_value = self.get_operand(operand);
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;


        operand_value |= carry << 8;
        carry = operand_value & 0b1;
        operand_value >>= 1;

        self.set_operand(operand, operand_value);

        Ok(carry)

    }

    fn rotate_right(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand) as u8;
        
        let carry = operand_value & 0b1;
        let result =  operand_value.wrapping_shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_left_arithmetic(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand) as u8;

        let result = operand_value.shl(1);
        let carry = result & 0b1u8;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_artithmetic(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand) as i8;
        
        let carry = operand_value & 0b1;
        let result =  operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_logical(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand) as u8;
        
        let carry = operand_value & 0b1;
        let result =  operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)    }

    fn swap(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);

        let result = operand_value.wrapping_shl(4);

        self.set_operand(operand, result );

        Ok(result)
    }

    fn call(&mut self, operand: &Operand) -> OperationResult<u16> {
        let next_instruction_address = self.cpu.get_pc() + 3;
        let call_address = self.get_operand(operand);

        self.push_to_stack(next_instruction_address);
        self.cpu.set_pc(call_address);
        

        Ok(next_instruction_address)
    }

    fn call_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let Operand::CC(condition) = operand0  else {
            return Err(InstructionError::InvalidOperandType { expected: OperandType::CC, received: (*operand0).into() })
        };

        if self.check_condition(condition) {
            self.call(operand1)
        } else {
            todo!("Adjust number of cycles taken")
        }
    }

    fn jump(&mut self, operand: &Operand) -> OperationResult<u16> {
        let address = self.get_operand(operand);

        self.cpu.set_pc(address);

        Ok(address)
    }

    fn jump_relative(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);
        let pc = self.cpu.get_pc();

        let result = operand_value + pc;
        self.cpu.set_pc(result);

        Ok(result)

    }

    fn jump_relative_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> OperationResult<u16> {
        let Operand::CC(condition) = operand0  else {
            return Err(InstructionError::InvalidOperandType { expected: OperandType::CC, received: (*operand0).into() })
        };

        if self.check_condition(condition) {
            self.jump_relative(operand1)
        } else {
            Ok(self.cpu.get_pc())     
        }
    }

    fn return_conditional(&mut self, operand: &Operand) -> OperationResult<u16> {
        let Operand::CC(condition) = operand  else {
            return Err(InstructionError::InvalidOperandType { expected: OperandType::CC, received: (*operand).into() })
        };

        if self.check_condition(condition) {
            self.ret()
        } else {
            todo!("Change cycles")
        }

    }

    fn call_vector(&mut self, operand: &Operand) -> OperationResult<u16> {
        self.call(operand)
    }

    fn ret(&mut self) -> OperationResult<u16> {
        let new_pc = self.pop_from_stack();
        self.cpu.set_pc(new_pc);

        Ok(new_pc)
    }

    fn return_from_interrupt(&mut self) -> OperationResult<u16> {
        self.enable_interrupts()?;
        self.ret()
    }

    fn complement_carry_flag(&mut self) -> OperationResult<u16> {
        let carry = self.cpu.get_flag(Flag::Carry);
        self.cpu.set_flag(Flag::Carry, !(carry != 0));

        Ok(0)
    }

    fn set_carry_flag(&mut self) -> OperationResult<u16> {
        self.cpu.set_flag(Flag::Carry, true);

        Ok(0)
    }

    fn pop(&mut self, operand: &Operand) -> OperationResult<u16> {
        let stack_value = self.pop_from_stack();

        self.set_operand(operand, stack_value);

        Ok(stack_value)
    }

    fn push(&mut self, operand: &Operand) -> OperationResult<u16> {
        let operand_value = self.get_operand(operand);

        self.push_to_stack(operand_value);

        Ok(operand_value)
    }

    fn disable_interrupts(&mut self) -> OperationResult<u16> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, false);

        Ok(0)
    }

    fn enable_interrupts(&mut self) -> OperationResult<u16> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, true);

        Ok(0)
    }

    fn halt(&mut self) -> OperationResult<u16> {
        todo!()
    }

    fn decimal_adjust_accumulator(&mut self) -> OperationResult<u16> {
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
                self.add(&Operand::N8(adjustment))
            }  
      }

    }

    fn nop(&mut self) -> OperationResult<u16> {
        Ok(0)
    }

    fn stop(&mut self) -> OperationResult<u16> {
        todo!()
    }


}

type OperationResult<T> = Result<T, InstructionError>;

pub enum InstructionError {
    InvalidOperandType {
        expected: OperandType,
        received: OperandType,
    },
    InvalidOperation(Vec<u8>),
}
pub enum OperandType {
    R8,
    R16,
    R16Stk,
    R16Mem,
    N8,
    N16,
    U3,
    CC,
}

impl From<Operand> for OperandType {
    fn from(value: Operand) -> Self {
        match value {
            Operand::R8(_) => Self::R8,
            Operand::R16(_) => Self::R16,
            Operand::R16Stk(_) => Self::R16Stk,
            Operand::R16Mem(_) => Self::R16Mem,
            Operand::N8(_) => Self::N8,
            Operand::N16(_) => Self::N16,
            Operand::U3(_) => Self::U3,
            Operand::CC(_) => Self::CC,
        }
    }
}