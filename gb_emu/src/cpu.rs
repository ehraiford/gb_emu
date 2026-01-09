use crate::{
    bus::{Bus, MemoryAccessError},
    helper_functions::concat_2_bytes,
    instructions::{Instruction, InstructionError, InstructionResult, OpCode, Operand},
};
use std::ops::{Shl, Shr};

#[derive(Default)]
pub struct Cpu {
    registers: [u16; 6],
    ime: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_flag(&self, flag: Flag) -> u8 {
        match flag {
            Flag::InterruptMasterEnable => self.ime.into(),
            _ => (self.get_af() >> flag.get_af_index()) as u8,
        }
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::InterruptMasterEnable => self.ime = value,
            _ => {
                let flag_index = flag.get_af_index();
                let flag_mask = !(0b1 << flag_index);
                let af = self.get_af();

                let masked_af = af & flag_mask;
                let result = masked_af | (value as u16) << flag_index;

                self.set_af(result);
            },
        }
    }

    pub fn get_a(&self) -> u8 {
        (self.get_af() >> 8) as u8
    }
    pub fn set_a(&mut self, new_a: u8) {
        let f = self.get_f() as u16;
        let a = (new_a as u16) << 8;

        self.set_af(a | f);
    }
    fn get_b(&self) -> u8 {
        (self.get_bc() >> 8) as u8
    }
    pub fn set_b(&mut self, new_b: u8) {
        let c = self.get_c() as u16;
        let b = (new_b as u16) << 8;

        self.set_bc(b | c);
    }
    fn get_d(&self) -> u8 {
        (self.get_de() >> 8) as u8
    }
    pub fn set_d(&mut self, new_d: u8) {
        let e = self.get_c() as u16;
        let d = (new_d as u16) << 8;

        self.set_bc(d | e);
    }
    fn get_h(&self) -> u8 {
        (self.get_hl() >> 8) as u8
    }
    pub fn set_h(&mut self, new_h: u8) {
        let l = self.get_c() as u16;
        let h = (new_h as u16) << 8;

        self.set_bc(h | l);
    }
    fn get_f(&self) -> u8 {
        (self.get_af() & 0xFF) as u8
    }
    fn set_f(&mut self, new_f: u8) {
        self.set_af((self.get_af() & 0xFF00) | new_f as u16)
    }
    fn get_c(&self) -> u8 {
        (self.get_bc() & 0xFF) as u8
    }
    fn set_c(&mut self, new_c: u8) {
        self.set_bc((self.get_bc() & 0xFF00) | new_c as u16)
    }
    fn get_e(&self) -> u8 {
        (self.get_de() & 0xFF) as u8
    }
    fn set_e(&mut self, new_e: u8) {
        self.set_de((self.get_de() & 0xFF00) | new_e as u16)
    }
    fn set_l(&mut self, new_l: u8) {
        self.set_hl((self.get_hl() & 0xFF00) | new_l as u16)
    }
    fn get_l(&self) -> u8 {
        (self.get_hl() & 0xFF) as u8
    }

    fn get_af(&self) -> u16 {
        self.registers[0]
    }
    fn set_af(&mut self, value: u16) {
        self.registers[0] = value;
    }

    fn get_bc(&self) -> u16 {
        self.registers[1]
    }
    fn set_bc(&mut self, value: u16) {
        self.registers[1] = value;
    }

    fn get_de(&self) -> u16 {
        self.registers[2]
    }
    fn set_de(&mut self, value: u16) {
        self.registers[2] = value;
    }
    fn get_hl(&self) -> u16 {
        self.registers[3]
    }
    fn set_hl(&mut self, val: u16) {
        self.registers[3] = val;
    }
    fn get_sp(&self) -> u16 {
        self.registers[4]
    }
    fn set_sp(&mut self, value: u16) {
        self.registers[4] = value;
    }
    pub fn get_pc(&self) -> u16 {
        self.registers[5]
    }
    pub fn set_pc(&mut self, value: u16) {
        self.registers[5] = value;
    }

    pub fn check_condition(&self, cond: &Condition) -> bool {
        self.get_condition(cond) == 1
    }

    fn get_condition(&self, cond: &Condition) -> u8 {
        match cond {
            Condition::NotZero => !self.get_flag(Flag::Zero) & 0b1,
            Condition::Zero => self.get_flag(Flag::Zero),
            Condition::NotCarry => !self.get_flag(Flag::Carry) & 0b1,
            Condition::Carry => self.get_flag(Flag::Carry),
        }
    }

    pub fn get_operand(&mut self, operand: &Operand, bus: &mut Bus, bytes: [u8; 3]) -> InstructionResult<u16> {
        match *operand {
            Operand::Immediate(imm) => Ok(imm as u16),
            Operand::A => Ok(self.get_a() as u16),
            Operand::AF => Ok(self.get_af()),
            Operand::B => Ok(self.get_b() as u16),
            Operand::BC => Ok(self.get_bc()),
            Operand::BCPointer => Ok(bus.read(self.get_bc())? as u16),
            Operand::C => Ok(self.get_c() as u16),
            Operand::D => Ok(self.get_d() as u16),
            Operand::DE => Ok(self.get_de()),
            Operand::DEPointer => Ok(bus.read(self.get_de())? as u16),
            Operand::E => Ok(self.get_e() as u16),
            Operand::H => Ok(self.get_h() as u16),
            Operand::HL => Ok(self.get_hl()),
            Operand::HLPointer => Ok(bus.read(self.get_hl())? as u16),
            Operand::L => Ok(self.get_l() as u16),
            Operand::NC => Ok(self.get_condition(&Condition::NotCarry) as u16),
            Operand::NZ => Ok(self.get_condition(&Condition::NotZero) as u16),
            Operand::SP => Ok(self.get_sp()),
            Operand::Z => Ok(self.get_condition(&Condition::Zero) as u16),
            Operand::A16 => Ok(concat_2_bytes(bytes[1], bytes[2])),
            Operand::A16Pointer => Ok(bus.read(concat_2_bytes(bytes[1], bytes[2]))? as u16),
            Operand::E8 => Ok(bytes[1] as u16),
            Operand::N16 => Ok(concat_2_bytes(bytes[1], bytes[2])),
            Operand::N8 => Ok(bytes[1] as u16),
            Operand::FF00OffsetByA => Ok(0xFF00 + self.get_a() as u16),
            Operand::FF00OffsetByC => Ok(0xFF00 + self.get_a() as u16),
            Operand::HLD => {
                let value = self.get_hl();
                self.set_hl(value - 1);
                Ok(value)
            },
            Operand::HLI => {
                let value = self.get_hl();
                self.set_hl(value + 1);
                Ok(value)
            },
            Operand::SPAndE8 => Ok(self.get_sp().wrapping_add(bytes[1] as i8 as u16)),
        }
    }

    pub fn set_operand(&mut self, operand: &Operand, value: u16, bus: &mut Bus) -> InstructionResult<()> {
        match operand {
            Operand::A => Ok(self.set_a(value as u8)),
            Operand::A16 => todo!(),
            Operand::A16Pointer => todo!(),
            Operand::AF => todo!(),
            Operand::B => Ok(self.set_b(value as u8)),
            Operand::BC => todo!(),
            Operand::BCPointer => todo!(),
            Operand::C => Ok(self.set_c(value as u8)),
            Operand::D => Ok(self.set_d(value as u8)),
            Operand::DE => todo!(),
            Operand::DEPointer => todo!(),
            Operand::E => Ok(self.set_e(value as u8)),
            Operand::E8 => Err(InstructionError::OperandCannotBeSet),
            Operand::FF00OffsetByA => Err(InstructionError::OperandCannotBeSet),
            Operand::FF00OffsetByC => Err(InstructionError::OperandCannotBeSet),
            Operand::H => Ok(self.set_h(value as u8)),
            Operand::HL => todo!(),
            Operand::HLD => Err(InstructionError::OperandCannotBeSet),
            Operand::HLI => Err(InstructionError::OperandCannotBeSet),
            Operand::HLPointer => todo!(),
            Operand::Immediate(_) => Err(InstructionError::OperandCannotBeSet),
            Operand::L => Ok(self.set_l(value as u8)),
            Operand::N16 => todo!(),
            Operand::N8 => todo!(),
            Operand::NC => todo!(),
            Operand::NZ => todo!(),
            Operand::SP => todo!(),
            Operand::SPAndE8 => Err(InstructionError::OperandCannotBeSet),
            Operand::Z => todo!(),
        }
    }
}

pub enum Flag {
    Zero,
    Subtraction,
    HalfCarry,
    Carry,
    InterruptMasterEnable,
}

impl Flag {
    fn get_af_index(&self) -> usize {
        match self {
            Flag::Zero => 7,
            Flag::Subtraction => 6,
            Flag::HalfCarry => 5,
            Flag::Carry => 4,
            _ => unreachable!("This function is invalid for any other falg and isn't called anywhere to reach this."),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

pub struct OperationContext<'a, 'b> {
    cpu: &'a mut Cpu,
    bus: &'b mut Bus,
    bytes: [u8; 3],
}

impl<'a, 'b> OperationContext<'a, 'b> {
    pub fn new(cpu: &'a mut Cpu, bus: &'b mut Bus, bytes: [u8; 3]) -> Self {
        Self { cpu, bus, bytes }
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

    fn get_condition(&self, operand: &Operand) -> InstructionResult<Condition> {
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

    fn get_operand(&mut self, operand: &Operand) -> InstructionResult<u16> {
        self.cpu.get_operand(operand, self.bus, self.bytes)
    }
    fn set_operand(&mut self, operand: &Operand, value: u16) {
        self.cpu.set_operand(operand, value, self.bus);
    }

    pub fn perform_instruction(&mut self, instruction: &Instruction) -> InstructionResult<()> {
        match instruction.op_code {
            OpCode::Adc => self.add_with_carry(&instruction.operands[0])?,
            OpCode::Add => self.add(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::And => self.and(&instruction.operands[0])?,
            OpCode::Bit => self.test_bit(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::Call => self.call(&instruction.operands[0])?,
            OpCode::Ccf => self.complement_carry_flag()?,
            OpCode::Cp => self.compare(&instruction.operands[0])?,
            OpCode::Cpl => self.complement()?,
            OpCode::Daa => self.decimal_adjust_accumulator()?,
            OpCode::Dec => self.decrement(&instruction.operands[0])?,
            OpCode::Di => self.disable_interrupts()?,
            OpCode::Ei => self.enable_interrupts()?,
            OpCode::Halt => self.halt()?,
            OpCode::Illegal => unreachable!(),
            OpCode::Inc => self.increment(&instruction.operands[0])?,
            OpCode::Jp => self.jump(&instruction.operands[0])?,
            OpCode::Jr => self.jump_relative(&instruction.operands[0])?,
            OpCode::Ld => self.load(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::Ldh => self.load_high(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::Nop => self.nop()?,
            OpCode::Or => self.or(&instruction.operands[0])?,
            OpCode::Pop => self.pop(&instruction.operands[0])?,
            OpCode::Prefix => unreachable!(),
            OpCode::Push => self.push(&instruction.operands[0])?,
            OpCode::Res => self.clear_bit(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::Ret => self.ret()?,
            OpCode::Reti => self.return_from_interrupt()?,
            OpCode::Rl => self.rotate_left_through_carry(&instruction.operands[0])?,
            OpCode::Rla => self.rotate_left_through_carry(&Operand::A)?,
            OpCode::Rlc => self.rotate_left(&instruction.operands[0])?,
            OpCode::Rlca => self.rotate_left(&Operand::A)?,
            OpCode::Rr => self.rotate_right_through_carry(&instruction.operands[0])?,
            OpCode::Rra => self.rotate_right_through_carry(&Operand::A)?,
            OpCode::Rrc => self.rotate_right(&instruction.operands[0])?,
            OpCode::Rrca => self.rotate_right(&Operand::A)?,
            OpCode::Rst => self.call_vector(&instruction.operands[0])?,
            OpCode::Sbc => self.subtract_with_carry(&instruction.operands[0])?,
            OpCode::Scf => self.set_carry_flag()?,
            OpCode::Set => self.set_bit(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::Sla => self.shift_left_arithmetic(&instruction.operands[0])?,
            OpCode::Sra => self.shift_right_artithmetic(&instruction.operands[0])?,
            OpCode::Srl => self.shift_right_logical(&instruction.operands[0])?,
            OpCode::Stop => self.stop()?,
            OpCode::Sub => self.subtract(&instruction.operands[0])?,
            OpCode::Swap => self.swap(&instruction.operands[0])?,
            OpCode::Xor => self.xor(&instruction.operands[0])?,
            OpCode::JpConditional => self.jump_conditional(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::JrConditional => {
                self.jump_relative_conditional(&instruction.operands[0], &instruction.operands[1])?
            },
            OpCode::CallConditional => self.call_conditional(&instruction.operands[0], &instruction.operands[1])?,
            OpCode::RetConditional => self.return_conditional(&instruction.operands[0])?,
        };

        Ok(())
    }

    fn load(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let value = self.get_operand(operand1)?;
        self.set_operand(operand0, value);

        Ok(value)
    }

    fn load_high(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let value = self.get_operand(operand1)?;

        if value < 0xFF00 {
            return Err(InstructionError::LdhLowValue(value));
        }
        self.set_operand(operand0, value);

        Ok(value)
    }

    fn add_with_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let carry = self.cpu.get_flag(Flag::Carry) as u16;
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;

        let result = operand_value + a + carry;
        self.set_a(result as u8);

        Ok(result)
    }

    fn add(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let operand0_val = self.get_operand(operand0)?;
        let operand1_val = self.get_operand(operand1)?;

        let result = operand0_val + operand1_val;
        self.set_operand(operand0, result);

        Ok(result)
    }

    fn compare(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;

        let result = a.wrapping_sub(operand_value);

        Ok(result)
    }

    fn decrement(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;

        let result = operand_value - 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn increment(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;

        let result = operand_value + 1;
        self.set_operand(operand, result);

        Ok(result)
    }

    fn subtract_with_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;
        let carry = self.cpu.get_flag(Flag::Carry) as u16;

        let result = a - carry - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn subtract(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;

        let result = a - operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn and(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
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
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;

        let result = a | operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn xor(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
        let a = self.get_a() as u16;

        let result = a ^ operand_value;
        self.set_a(result as u8);

        Ok(result)
    }

    fn test_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0)?;
        let test_value = self.get_operand(operand1)?;

        let result = (test_value >> index) & 0b1;

        Ok(result)
    }

    fn clear_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0)?;
        let mask = !(0b1 << index);
        let operand_value = self.get_operand(operand1)?;

        let result = operand_value & mask;
        self.set_operand(operand1, result);

        Ok(result)
    }

    fn set_bit(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let index = self.get_operand(operand0)?;
        let mask = 0b1 << index;
        let operand_value = self.get_operand(operand1)?;

        let result = operand_value | mask;
        self.set_operand(operand1, result);

        Ok(result)
    }

    fn rotate_left_through_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let mut operand_value = self.get_operand(operand)?;
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;

        operand_value <<= 1;
        operand_value |= carry;
        carry = operand_value >> 8;

        self.set_operand(operand, operand_value);

        Ok(carry)
    }

    fn rotate_left(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)? as u8;

        let result = operand_value.wrapping_shl(1);
        let carry = result & 0b1;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn rotate_right_through_carry(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let mut operand_value = self.get_operand(operand)?;
        let mut carry = self.cpu.get_flag(Flag::Carry) as u16;

        operand_value |= carry << 8;
        carry = operand_value & 0b1;
        operand_value >>= 1;

        self.set_operand(operand, operand_value);

        Ok(carry)
    }

    fn rotate_right(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)? as u8;

        let carry = operand_value & 0b1;
        let result = operand_value.wrapping_shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_left_arithmetic(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)? as u8;

        let result = operand_value.shl(1);
        let carry = result & 0b1u8;

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_artithmetic(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)? as i8;

        let carry = operand_value & 0b1;
        let result = operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn shift_right_logical(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)? as u8;

        let carry = operand_value & 0b1;
        let result = operand_value.shr(1);

        self.set_operand(operand, result as u16);

        Ok(carry as u16)
    }

    fn swap(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;

        let result = operand_value.wrapping_shl(4);

        self.set_operand(operand, result);

        Ok(result)
    }

    fn call(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let next_instruction_address = self.cpu.get_pc() + 3;
        let call_address = self.get_operand(operand)?;

        self.push_to_stack(next_instruction_address);
        self.cpu.set_pc(call_address);

        Ok(next_instruction_address)
    }

    fn call_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let condition: Condition = self.get_condition(operand0)?;
        if self.check_condition(&condition) {
            self.call(operand1)
        } else {
            todo!("Adjust number of cycles taken")
        }
    }

    fn jump(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let address = self.get_operand(operand)?;

        self.cpu.set_pc(address);

        Ok(address)
    }

    fn jump_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let condition: Condition = self.get_condition(operand0)?;
        if self.check_condition(&condition) {
            self.jump(operand1)
        } else {
            Ok(self.cpu.get_pc())
        }
    }

    fn jump_relative(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let operand_value = self.get_operand(operand)?;
        let pc = self.cpu.get_pc();

        let result = operand_value + pc;
        self.cpu.set_pc(result);

        Ok(result)
    }

    fn jump_relative_conditional(&mut self, operand0: &Operand, operand1: &Operand) -> InstructionResult<u16> {
        let condition = self.get_condition(operand0)?;
        if self.check_condition(&condition) {
            self.jump_relative(operand1)
        } else {
            Ok(self.cpu.get_pc())
        }
    }

    fn return_conditional(&mut self, operand: &Operand) -> InstructionResult<u16> {
        let condition = self.get_condition(operand)?;
        if self.check_condition(&condition) {
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
        let operand_value = self.get_operand(operand)?;

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
                self.subtract(&Operand::Immediate(adjustment))
            },
            _ => {
                let a = self.get_a();
                if self.cpu.get_flag(Flag::HalfCarry) == 1 || a & 0xF > 0x9 {
                    adjustment += 0x6;
                }
                if self.cpu.get_flag(Flag::Carry) == 1 || a > 0x99 {
                    adjustment += 0x60;
                }
                self.add(&Operand::A, &Operand::Immediate(adjustment))
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
            let instruction_result = <&Instruction>::try_from(bytes);

            // the try_from should only fail on the invalid opcodes
            if instruction_result.is_err() {
                assert!(INVALID_BYTES.contains(&bytes[0]));
            }
        }
    }
}
