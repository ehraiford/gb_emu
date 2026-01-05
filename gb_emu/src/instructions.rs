use crate::{bus::Bus, cpu::{Cpu, Flag, R8, R16, R16Mem, R16Stk}};


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
    TestBit(Operand),
    ClearBit(Operand),
    SetBit(Operand),
    RotateLeftThroughCarry(Operand),
    RotateLeft(Operand),
    RotateRightThroughCarry(Operand),
    RotateRight(Operand),
    ShiftLeftArithmetic(Operand),
    ShiftRightArtithmetic(Operand),
    ShiftRightLogical(Operand),
    Swap(Operand),
    Call(Operand),
    CallConditional(Operand),
    Jump(Operand),
    JumpRelative(Operand),
    JumpRelativeConditional(Operand),
    Return,
    ReturnConditional(Operand),
    ReturnFromInterrupt,
    CallVector(Operand),
    ComplementCarryFlag,
    SetCarryFlag,
    Pop,
    Push,
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
            Operation::Subtract(operand) => todo!(),
            Operation::And(operand) => todo!(),
            Operation::Complement => todo!(),
            Operation::Or(operand) => todo!(),
            Operation::Xor(operand) => todo!(),
            Operation::TestBit(operand) => todo!(),
            Operation::ClearBit(operand) => todo!(),
            Operation::SetBit(operand) => todo!(),
            Operation::RotateLeftThroughCarry(operand) => todo!(),
            Operation::RotateLeft(operand) => todo!(),
            Operation::RotateRightThroughCarry(operand) => todo!(),
            Operation::RotateRight(operand) => todo!(),
            Operation::ShiftLeftArithmetic(operand) => todo!(),
            Operation::ShiftRightArtithmetic(operand) => todo!(),
            Operation::ShiftRightLogical(operand) => todo!(),
            Operation::Swap(operand) => todo!(),
            Operation::Call(operand) => todo!(),
            Operation::CallConditional(operand) => todo!(),
            Operation::Jump(operand) => todo!(),
            Operation::JumpRelative(operand) => todo!(),
            Operation::JumpRelativeConditional(operand) => todo!(),
            Operation::Return => todo!(),
            Operation::ReturnConditional(operand) => todo!(),
            Operation::ReturnFromInterrupt => todo!(),
            Operation::CallVector(operand) => todo!(),
            Operation::ComplementCarryFlag => todo!(),
            Operation::SetCarryFlag => todo!(),
            Operation::Pop => todo!(),
            Operation::Push => todo!(),
            Operation::DisableInterrupts => todo!(),
            Operation::EnableInterrupts => todo!(),
            Operation::Halt => todo!(),
            Operation::DecimalAdjustAccumulator => todo!(),
            Operation::Nop => todo!(),
            Operation::Stop => todo!(),
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


}

type OperationResult<T> = Result<T, OperationError>;

pub enum OperationError {

}