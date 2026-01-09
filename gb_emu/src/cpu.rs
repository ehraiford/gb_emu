use crate::{
    bus::{Bus, MemoryAccessResult},
    helper_functions::concat_2_bytes,
    instructions::{
        EightBitOperand, Instruction, InstructionError, InstructionOutcome, InstructionResult, OpCode, Operand,
        OperandType, SixteenBitOperand,
    },
};

#[derive(Default)]
pub struct Cpu {
    registers: [u16; 6],
    ime: bool,
}

impl Cpu {
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        let mut new_flags = z as u8;
        new_flags = (new_flags << 1) | n as u8;
        new_flags = (new_flags << 1) | h as u8;
        new_flags = (new_flags << 1) | c as u8;

        self.set_f(new_flags);
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::InterruptMasterEnable => self.ime.into(),
            _ => (self.get_af() >> flag.get_af_index()) & 0b1 == 1,
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
        let e = self.get_e() as u16;
        let d = (new_d as u16) << 8;

        self.set_de(d | e);
    }
    fn get_h(&self) -> u8 {
        (self.get_hl() >> 8) as u8
    }
    pub fn set_h(&mut self, new_h: u8) {
        let l = self.get_l() as u16;
        let h = (new_h as u16) << 8;

        self.set_hl(h | l);
    }
    fn get_f(&self) -> u8 {
        (self.get_af() & 0xFF) as u8
    }
    fn set_f(&mut self, new_f: u8) {
        self.set_af((self.get_af() & 0xFF00) | (new_f as u16) & 0xF0)
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
    fn set_hl(&mut self, value: u16) {
        self.registers[3] = value;
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
        match cond {
            Condition::NotZero => !self.get_flag(Flag::Zero),
            Condition::Zero => self.get_flag(Flag::Zero),
            Condition::NotCarry => !self.get_flag(Flag::Carry),
            Condition::Carry => self.get_flag(Flag::Carry),
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
            _ => unreachable!("This function is invalid for any other flag and isn't called anywhere to reach this."),
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

impl TryFrom<Operand> for Condition {
    type Error = InstructionError;

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::Carry => Ok(Condition::Carry),
            Operand::NotCarry => Ok(Condition::NotCarry),
            Operand::NotZero => Ok(Condition::NotZero),
            Operand::Zero => Ok(Condition::Zero),
            _ => Err(InstructionError::InvalidOperand),
        }
    }
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

    fn push_to_stack(&mut self, value: u16) -> MemoryAccessResult<()> {
        let new_sp = self.cpu.get_sp().wrapping_sub(2);
        self.cpu.set_sp(new_sp);
        self.bus.write_u16(new_sp, value)
    }

    fn pop_from_stack(&mut self) -> MemoryAccessResult<u16> {
        let sp = self.cpu.get_sp();
        let value = self.bus.read_u16(sp)?;
        self.cpu.set_sp(sp.wrapping_add(2));
        Ok(value)
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

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.cpu.set_flags(z, n, h, c)
    }

    fn get_u8_operand(&mut self, operand: &EightBitOperand) -> MemoryAccessResult<u8> {
        match operand {
            EightBitOperand::A => Ok(self.cpu.get_a()),
            EightBitOperand::B => Ok(self.cpu.get_b()),
            EightBitOperand::L => Ok(self.cpu.get_l()),
            EightBitOperand::C => Ok(self.cpu.get_c()),
            EightBitOperand::D => Ok(self.cpu.get_d()),
            EightBitOperand::E => Ok(self.cpu.get_e()),
            EightBitOperand::H => Ok(self.cpu.get_h()),
            EightBitOperand::HLPointer => Ok(self.bus.read(self.cpu.get_hl())?),
            EightBitOperand::BCPointer => Ok(self.bus.read(self.cpu.get_bc())?),
            EightBitOperand::DEPointer => Ok(self.bus.read(self.cpu.get_de())?),
            EightBitOperand::A16Pointer => Ok(self.bus.read(concat_2_bytes(self.bytes[1], self.bytes[2]))?),
            EightBitOperand::FF00OffsetByA => Ok(self.bus.read(0xFF00 + self.cpu.get_a() as u16)?),
            EightBitOperand::FF00OffsetByC => Ok(self.bus.read(0xFF00 + self.cpu.get_c() as u16)?),
            EightBitOperand::N8 => Ok(self.bytes[1]),
            EightBitOperand::Immediate(val) => Ok(*val),
            EightBitOperand::HLIPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_add(1));
                Ok(self.bus.read(hl)?)
            },
            EightBitOperand::HLDPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_sub(1));
                Ok(self.bus.read(hl)?)
            },
        }
    }

    fn set_u8_operand(&mut self, operand: &EightBitOperand, value: u8) -> InstructionResult<()> {
        match operand {
            EightBitOperand::A => Ok(self.cpu.set_a(value)),
            EightBitOperand::A16Pointer => todo!(),
            EightBitOperand::B => Ok(self.cpu.set_b(value)),
            EightBitOperand::HLIPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_add(1));
                Ok(self.bus.write(hl, value)?)
            },
            EightBitOperand::HLDPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_sub(1));
                Ok(self.bus.write(hl, value)?)
            },
            EightBitOperand::HLPointer => Ok(self.bus.write(self.cpu.get_hl(), value)?),
            EightBitOperand::DEPointer => Ok(self.bus.write(self.cpu.get_de(), value)?),
            EightBitOperand::BCPointer => Ok(self.bus.write(self.cpu.get_bc(), value)?),
            EightBitOperand::L => Ok(self.cpu.set_l(value)),
            EightBitOperand::C => Ok(self.cpu.set_c(value)),
            EightBitOperand::D => Ok(self.cpu.set_d(value)),
            EightBitOperand::E => Ok(self.cpu.set_e(value)),
            EightBitOperand::FF00OffsetByA => Ok(self.bus.write(0xFF00 + self.cpu.get_a() as u16, value)?),
            EightBitOperand::FF00OffsetByC => Ok(self.bus.write(0xFF00 + self.cpu.get_c() as u16, value)?),
            EightBitOperand::H => Ok(self.cpu.set_h(value)),
            _ => Err(InstructionError::InvalidOperand),
        }
    }

    fn get_u16_operand(&mut self, operand: &SixteenBitOperand) -> u16 {
        match operand {
            SixteenBitOperand::BC => self.cpu.get_bc(),
            SixteenBitOperand::DE => self.cpu.get_de(),
            SixteenBitOperand::HL => self.cpu.get_hl(),
            SixteenBitOperand::AF => self.cpu.get_af(),
            SixteenBitOperand::SP => self.cpu.get_sp(),
            SixteenBitOperand::A16 => concat_2_bytes(self.bytes[1], self.bytes[2]),
            SixteenBitOperand::N16 => concat_2_bytes(self.bytes[1], self.bytes[2]),
            SixteenBitOperand::Immediate(imm) => *imm,
        }
    }

    fn set_u16_operand(&mut self, operand: &SixteenBitOperand, value: u16) -> InstructionResult<()> {
        match operand {
            SixteenBitOperand::BC => Ok(self.cpu.set_bc(value)),
            SixteenBitOperand::DE => Ok(self.cpu.set_de(value)),
            SixteenBitOperand::HL => Ok(self.cpu.set_hl(value)),
            SixteenBitOperand::AF => Ok(self.cpu.set_af(value)),
            SixteenBitOperand::SP => Ok(self.cpu.set_sp(value)),
            SixteenBitOperand::N16 => Err(InstructionError::InvalidOperand),
            SixteenBitOperand::A16 => Err(InstructionError::InvalidOperand),
            SixteenBitOperand::Immediate(_) => Err(InstructionError::InvalidOperand),
        }
    }

    pub fn perform_instruction(&mut self, instruction: &Instruction) -> InstructionResult<InstructionOutcome> {
        match instruction.op_code {
            OpCode::Adc => self.add_with_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Add => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.add_8_bit(&EightBitOperand::try_from(instruction.operands[0]).unwrap())
                },
                OperandType::SixteenBitOperand => self.add_16_bit(
                    &SixteenBitOperand::try_from(instruction.operands[0]).unwrap(),
                    &SixteenBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                _ => return Err(InstructionError::InvalidOperand),
            },
            OpCode::And => self.and(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Bit => self.bit(
                &&EightBitOperand::try_from(instruction.operands[0])?,
                &&EightBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Call => self.call(&SixteenBitOperand::try_from(instruction.operands[0])?),
            OpCode::CallConditional => self.call_conditional(
                &Condition::try_from(instruction.operands[0])?,
                &SixteenBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Ccf => self.complement_carry_flag(),
            OpCode::Cp => self.compare(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Cpl => self.complement(),
            OpCode::Daa => self.decimal_adjust_accumulator(),
            OpCode::Dec => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.decrement_8_bit(&EightBitOperand::try_from(instruction.operands[0])?)
                },
                OperandType::SixteenBitOperand => {
                    self.decrement_16_bit(&SixteenBitOperand::try_from(instruction.operands[0])?)
                },
                _ => return Err(InstructionError::InvalidOperand),
            },
            OpCode::Di => self.disable_interrupts(),
            OpCode::Ei => self.enable_interrupts(),
            OpCode::Halt => self.halt(),
            OpCode::Illegal => unreachable!("We should have caught this before here."),
            OpCode::Inc => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.increment_8_bit(&EightBitOperand::try_from(instruction.operands[0])?)
                },
                OperandType::SixteenBitOperand => {
                    self.increment_16_bit(&SixteenBitOperand::try_from(instruction.operands[0])?)
                },
                _ => return Err(InstructionError::InvalidOperand),
            },
            OpCode::Jp => self.jump(&SixteenBitOperand::try_from(instruction.operands[0])?),
            OpCode::JpConditional => self.jump_conditional(
                &Condition::try_from(instruction.operands[0])?,
                &SixteenBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Jr => self.jump_relative(self.bytes[1] as i8),
            OpCode::JrConditional => {
                self.jump_relative_conditional(&Condition::try_from(instruction.operands[0])?, self.bytes[1] as i8)
            },
            OpCode::Ld => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => self.load_8_bit(
                    &EightBitOperand::try_from(instruction.operands[0])?,
                    &EightBitOperand::try_from(instruction.operands[1])?,
                ),
                OperandType::SixteenBitOperand => self.load_16_bit(
                    &SixteenBitOperand::try_from(instruction.operands[0])?,
                    &&SixteenBitOperand::try_from(instruction.operands[1])?,
                ),
                _ => return Err(InstructionError::InvalidOperand),
            },
            OpCode::Ldh => self.load_high(
                &EightBitOperand::try_from(instruction.operands[0])?,
                &EightBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Nop => self.nop(),
            OpCode::Or => self.or(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Pop => self.pop(&SixteenBitOperand::try_from(instruction.operands[0])?),
            OpCode::Prefix => unreachable!("This should have been caught prior"),
            OpCode::Push => self.push(&SixteenBitOperand::try_from(instruction.operands[0])?),
            OpCode::Res => self.clear_bit(
                &EightBitOperand::try_from(instruction.operands[0])?,
                &EightBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Ret => self.ret(),
            OpCode::RetConditional => self.return_conditional(&Condition::try_from(instruction.operands[0])?),
            OpCode::Reti => self.return_from_interrupt(),
            OpCode::Rl => self.rotate_left_through_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Rla => self.rotate_left_through_carry_a(),
            OpCode::Rlc => self.rotate_left_into_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Rlca => self.rotate_left_into_carry_a(),
            OpCode::Rr => self.rotate_right_through_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Rra => self.rotate_right_through_carry_a(),
            OpCode::Rrc => self.rotate_right_into_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Rrca => self.rotate_right_into_carry_a(),
            OpCode::Rst => self.call_vector(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Sbc => self.subtract_with_carry(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Scf => self.set_carry_flag(),
            OpCode::Set => self.set_bit(
                &EightBitOperand::try_from(instruction.operands[0])?,
                &EightBitOperand::try_from(instruction.operands[1])?,
            ),
            OpCode::Sla => self.shift_left_arithmetically(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Sra => self.shift_right_arithmetically(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Srl => self.shift_right_logically(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Stop => self.stop(),
            OpCode::Sub => self.subtract(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Swap => self.swap(&EightBitOperand::try_from(instruction.operands[0])?),
            OpCode::Xor => self.xor(&EightBitOperand::try_from(instruction.operands[0])?),
        }
    }

    fn add_with_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand)?;
        let carry = self.cpu.get_flag(Flag::Carry);

        let (middle_result, overflowed_middle) = a.overflowing_add(operand_value);
        let (result, overflowed_end) = middle_result.overflowing_add(carry as u8);

        self.set_a(result);
        self.set_flags(
            result == 0,
            false,
            bit_3_overflow(vec![a, operand_value, carry as u8]),
            overflowed_middle | overflowed_end,
        );

        Ok(InstructionOutcome::Ok)
    }

    fn add_8_bit(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand)?;

        let (result, overflowed) = a.overflowing_add(operand_value);

        self.set_a(result);
        self.set_flags(result == 0, false, bit_3_overflow(vec![a, operand_value]), overflowed);

        Ok(InstructionOutcome::Ok)
    }

    fn add_16_bit(
        &mut self,
        operand0: &SixteenBitOperand,
        operand1: &SixteenBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let operand0_value = self.get_u16_operand(operand0);
        let operand1_value = self.get_u16_operand(operand1);

        let (result, overflowed) = operand0_value.overflowing_add(operand1_value);

        self.set_u16_operand(operand0, result)?;
        self.set_flags(
            self.cpu.get_flag(Flag::Zero),
            false,
            bit_7_overflow(vec![operand0_value, operand1_value]),
            overflowed,
        );

        Ok(InstructionOutcome::Ok)
    }

    fn and(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(&operand)?;
        let a = self.get_a();

        let result = operand_value & a;

        self.set_a(result);
        self.set_flags(result == 0, false, true, false);

        Ok(InstructionOutcome::Ok)
    }

    fn bit(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let test_bit = self.get_u8_operand(operand0)?;
        let byte = self.get_u8_operand(operand1)?;

        let result = (byte >> test_bit) & 0b1;

        self.set_flags(result == 0, false, true, self.cpu.get_flag(Flag::Carry));

        Ok(InstructionOutcome::Ok)
    }

    fn call(&mut self, address: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let next_address = self.cpu.get_pc() + 3;
        self.push_to_stack(next_address)?;

        self.jump(address)
    }

    fn call_conditional(
        &mut self,
        condition: &Condition,
        address: &SixteenBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        if self.check_condition(condition) {
            self.call(address)?;
            Ok(InstructionOutcome::ExtraCycles(3))
        } else {
            Ok(InstructionOutcome::Ok)
        }
    }

    fn complement_carry_flag(&mut self) -> InstructionResult<InstructionOutcome> {
        let carry = self.cpu.get_flag(Flag::Carry);
        self.cpu.set_flag(Flag::Carry, !carry);

        Ok(InstructionOutcome::Ok)
    }

    fn compare(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(&operand)?;
        let a = self.get_a();

        self.set_flags(
            a == operand_value,
            true,
            bit_4_borrow(a, operand_value, false),
            a < operand_value,
        );

        Ok(InstructionOutcome::Ok)
    }

    fn complement(&mut self) -> InstructionResult<InstructionOutcome> {
        let a = self.get_a();
        self.set_a(!a);

        self.set_flags(
            self.cpu.get_flag(Flag::Zero),
            true,
            true,
            self.cpu.get_flag(Flag::Carry),
        );

        Ok(InstructionOutcome::Ok)
    }

    fn decimal_adjust_accumulator(&mut self) -> InstructionResult<InstructionOutcome> {
        let mut adjustment = 0;
        match self.cpu.get_flag(Flag::Subtraction) {
            true => {
                if self.cpu.get_flag(Flag::HalfCarry) {
                    adjustment += 0x6;
                }
                if self.cpu.get_flag(Flag::Carry) {
                    adjustment += 0x60;
                }
                self.subtract(&EightBitOperand::Immediate(adjustment))
            },
            false => {
                let a = self.get_a();
                if self.cpu.get_flag(Flag::HalfCarry) || a & 0xF > 0x9 {
                    adjustment += 0x6;
                }
                if self.cpu.get_flag(Flag::Carry) || a > 0x99 {
                    adjustment += 0x60;
                }
                self.add_8_bit(&EightBitOperand::Immediate(adjustment))
            },
        }
    }

    fn decrement_8_bit(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(operand)?;

        let result = operand_value.wrapping_sub(1);

        self.set_u8_operand(operand, result)?;
        self.set_flags(
            result == 0,
            true,
            bit_4_borrow(operand_value, 1, false),
            self.cpu.get_flag(Flag::Carry),
        );

        Ok(InstructionOutcome::Ok)
    }

    fn decrement_16_bit(&mut self, operand: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u16_operand(operand);

        let result = operand_value.wrapping_sub(1);

        self.set_u16_operand(operand, result)?;

        Ok(InstructionOutcome::Ok)
    }

    fn increment_8_bit(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(operand)?;

        let result = operand_value.wrapping_add(1);

        self.set_u8_operand(operand, result)?;
        self.set_flags(
            result == 0,
            false,
            bit_3_overflow(vec![operand_value, 1]),
            self.cpu.get_flag(Flag::Carry),
        );

        Ok(InstructionOutcome::Ok)
    }

    fn increment_16_bit(&mut self, operand: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u16_operand(operand);

        let result = operand_value.wrapping_add(1);

        self.set_u16_operand(operand, result)?;

        Ok(InstructionOutcome::Ok)
    }

    fn jump(&mut self, operand: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let address = self.get_u16_operand(operand);
        self.cpu.set_pc(address);

        Ok(InstructionOutcome::Ok)
    }

    fn jump_conditional(
        &mut self,
        condition: &Condition,
        address: &SixteenBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        if self.check_condition(condition) {
            self.jump(address)?;
            Ok(InstructionOutcome::ExtraCycles(1))
        } else {
            Ok(InstructionOutcome::Ok)
        }
    }

    fn jump_relative(&mut self, offset: i8) -> InstructionResult<InstructionOutcome> {
        let offset = offset as i16;
        let next_address = (self.cpu.get_pc() + 2) as i16;

        let jump_address = offset + next_address;

        self.cpu.set_pc(jump_address as u16);

        Ok(InstructionOutcome::Ok)
    }

    fn jump_relative_conditional(
        &mut self,
        condition: &Condition,
        offset: i8,
    ) -> InstructionResult<InstructionOutcome> {
        if self.check_condition(condition) {
            self.jump_relative(offset)?;
            Ok(InstructionOutcome::ExtraCycles(1))
        } else {
            Ok(InstructionOutcome::Ok)
        }
    }

    fn load_8_bit(
        &mut self,
        operand0: &EightBitOperand,
        operand1: &EightBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand1)?;
        self.set_u8_operand(operand0, value)?;

        Ok(InstructionOutcome::Ok)
    }

    fn load_16_bit(
        &mut self,
        operand0: &SixteenBitOperand,
        operand1: &SixteenBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u16_operand(operand1);
        self.set_u16_operand(operand0, value)?;

        Ok(InstructionOutcome::Ok)
    }

    fn load_high(
        &mut self,
        operand0: &EightBitOperand,
        operand1: &EightBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand1)?;
        self.set_u8_operand(operand0, value)?;

        Ok(InstructionOutcome::Ok)
    }

    fn or(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;
        let a = self.get_a();

        let result = value | a;

        self.set_a(result);
        self.cpu.set_flags(result == 0, false, false, false);

        Ok(InstructionOutcome::Ok)
    }

    fn pop(&mut self, operand: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.pop_from_stack()?;

        self.set_u16_operand(operand, value)?;

        Ok(InstructionOutcome::Ok)
    }

    fn push(&mut self, operand: &SixteenBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u16_operand(operand);

        self.push_to_stack(value)?;

        Ok(InstructionOutcome::Ok)
    }

    fn clear_bit(
        &mut self,
        operand0: &EightBitOperand,
        operand1: &EightBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let bit_number = self.get_u8_operand(operand0)?;
        let byte = self.get_u8_operand(operand1)?;

        let mask = !(1 << bit_number);
        let result = byte & mask;

        self.set_u8_operand(operand1, result)?;

        Ok(InstructionOutcome::Ok)
    }

    fn ret(&mut self) -> InstructionResult<InstructionOutcome> {
        let new_pc = self.pop_from_stack()?;
        self.cpu.set_pc(new_pc);

        Ok(InstructionOutcome::Ok)
    }

    fn return_conditional(&mut self, condition: &Condition) -> InstructionResult<InstructionOutcome> {
        if self.check_condition(condition) {
            self.ret()?;
            Ok(InstructionOutcome::ExtraCycles(3))
        } else {
            Ok(InstructionOutcome::Ok)
        }
    }

    fn return_from_interrupt(&mut self) -> InstructionResult<InstructionOutcome> {
        self.enable_interrupts()?;
        self.ret()
    }

    fn rotate_left_through_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;
        let carry = self.cpu.get_flag(Flag::Carry);

        let new_carry = (value >> 7) == 1;
        let result = (value << 1) | carry as u8;

        self.set_u8_operand(operand, result)?;

        self.cpu.set_flags(result == 0, false, false, new_carry);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_left_through_carry_a(&mut self) -> InstructionResult<InstructionOutcome> {
        self.rotate_left_through_carry(&EightBitOperand::A)?;

        self.cpu.set_flag(Flag::Zero, false);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_left_into_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;

        let new_carry = value >> 7;
        let result = (value << 1) | new_carry;

        self.set_u8_operand(operand, result)?;

        self.cpu.set_flags(result == 0, false, false, new_carry == 1);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_left_into_carry_a(&mut self) -> InstructionResult<InstructionOutcome> {
        self.rotate_left_into_carry(&EightBitOperand::A)?;

        self.cpu.set_flag(Flag::Zero, false);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_right_through_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;
        let carry = (self.cpu.get_flag(Flag::Carry) as u8) << 7;

        let new_carry = (value & 1) == 1;
        let result = (value >> 1) | carry;

        self.set_u8_operand(operand, result)?;

        self.cpu.set_flags(result == 0, false, false, new_carry);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_right_through_carry_a(&mut self) -> InstructionResult<InstructionOutcome> {
        self.rotate_right_through_carry(&EightBitOperand::A)?;

        self.cpu.set_flag(Flag::Zero, false);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_right_into_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;

        let new_carry = value << 7;
        let result = (value >> 1) | new_carry;

        self.set_u8_operand(operand, result)?;

        self.cpu.set_flags(result == 0, false, false, new_carry != 0);

        Ok(InstructionOutcome::Ok)
    }

    fn rotate_right_into_carry_a(&mut self) -> InstructionResult<InstructionOutcome> {
        self.rotate_right_into_carry(&EightBitOperand::A)?;

        self.cpu.set_flag(Flag::Zero, false);

        Ok(InstructionOutcome::Ok)
    }

    fn call_vector(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)? as u16;
        self.call(&SixteenBitOperand::Immediate(value))
    }

    fn subtract_with_carry(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand)?;
        let carry = self.cpu.get_flag(Flag::Carry);

        let (middle_result, overflowed_middle) = a.overflowing_sub(operand_value);
        let (result, overflowed_end) = middle_result.overflowing_sub(carry as u8);

        self.set_a(result);
        self.set_flags(
            result == 0,
            true,
            bit_4_borrow(a, operand_value, carry),
            overflowed_middle | overflowed_end,
        );

        Ok(InstructionOutcome::Ok)
    }

    fn set_carry_flag(&mut self) -> InstructionResult<InstructionOutcome> {
        self.cpu.set_flag(Flag::Carry, true);

        self.set_flags(self.cpu.get_flag(Flag::Zero), false, false, true);

        Ok(InstructionOutcome::Ok)
    }

    fn set_bit(
        &mut self,
        operand0: &EightBitOperand,
        operand1: &EightBitOperand,
    ) -> InstructionResult<InstructionOutcome> {
        let bit_number = self.get_u8_operand(operand0)?;
        let byte = self.get_u8_operand(operand1)?;

        let mask = 1 << bit_number;
        let result = byte | mask;

        self.set_u8_operand(operand1, result)?;

        Ok(InstructionOutcome::Ok)
    }

    fn shift_left_arithmetically(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;

        let new_carry = value >> 7 == 1;
        let result = value << 1;

        self.set_u8_operand(operand, result)?;
        self.cpu.set_flags(result == 0, false, false, new_carry);

        Ok(InstructionOutcome::Ok)
    }

    fn shift_right_arithmetically(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;
        let sign = value >> 7;

        let new_carry = value & 1 == 1;
        let result = (value >> 1) | sign;

        self.set_u8_operand(operand, result)?;
        self.cpu.set_flags(result == 0, false, false, new_carry);

        Ok(InstructionOutcome::Ok)
    }

    fn shift_right_logically(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let value = self.get_u8_operand(operand)?;

        let new_carry = value & 1 == 1;
        let result = value >> 1;

        self.set_u8_operand(operand, result)?;
        self.cpu.set_flags(result == 0, false, false, new_carry);

        Ok(InstructionOutcome::Ok)
    }

    fn subtract(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand)?;

        let (result, borrowed) = a.overflowing_sub(operand_value);

        self.set_a(result);
        self.set_flags(result == 0, true, bit_4_borrow(a, operand_value, false), borrowed);

        Ok(InstructionOutcome::Ok)
    }

    fn swap(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(operand)?;

        let result = operand_value.rotate_left(4);

        self.set_u8_operand(operand, result)?;
        self.set_flags(result == 0, false, false, false);

        Ok(InstructionOutcome::Ok)
    }

    fn xor(&mut self, operand: &EightBitOperand) -> InstructionResult<InstructionOutcome> {
        let operand_value = self.get_u8_operand(operand)?;
        let a = self.get_a();

        let result = a ^ operand_value;

        self.set_a(result);
        self.cpu.set_flags(result == 0, false, false, false);

        Ok(InstructionOutcome::Ok)
    }

    fn disable_interrupts(&mut self) -> InstructionResult<InstructionOutcome> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, false);

        Ok(InstructionOutcome::Ok)
    }

    fn enable_interrupts(&mut self) -> InstructionResult<InstructionOutcome> {
        self.cpu.set_flag(Flag::InterruptMasterEnable, true);

        Ok(InstructionOutcome::Ok)
    }

    fn halt(&mut self) -> InstructionResult<InstructionOutcome> {
        Ok(InstructionOutcome::Halt)
    }

    fn nop(&self) -> InstructionResult<InstructionOutcome> {
        Ok(InstructionOutcome::Ok)
    }

    fn stop(&mut self) -> InstructionResult<InstructionOutcome> {
        Ok(InstructionOutcome::Stop)
    }
}

fn bit_4_borrow(operand0: u8, operand1: u8, carry: bool) -> bool {
    (operand0 & 0x0F) < ((operand1 & 0x0F) + carry as u8)
}

fn bit_3_overflow(operands: Vec<u8>) -> bool {
    let mut result = 0;
    operands.iter().for_each(|o| result += o & 0x0F);
    result > 0x0F
}

fn bit_7_overflow(operands: Vec<u16>) -> bool {
    let mut result = 0;
    operands.iter().for_each(|o| result += o & 0xFF);
    result > 0xFF
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
