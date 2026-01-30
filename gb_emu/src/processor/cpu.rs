use crate::{
    bus::Bus,
    game_boy::{GameBoyEvent, GameBoyMode, TCycles, notate_event},
    helpers::log,
    io_devices::interrupts::Interrupt,
    processor::instructions::{
        EightBitOperand, Instruction, InstructionError, InstructionOutcome, OpCode, Operand, OperandType,
        SignedEightBitOperand, SixteenBitOperand,
    },
};

pub struct Cpu {
    registers: [u16; 6],
    ime: bool,
}

impl Cpu {
    pub fn tick(&mut self, bus: &mut Bus) -> TCycles {
        let pc = self.get_pc();

        let mut operation_context = CpuOperationContext::new(self, bus);

        if let Some(interrupt) = operation_context.try_get_interrupt() {
            return operation_context.handle_interrupt(interrupt);
        }

        let instruction = operation_context.bus.read_next_instruction(pc);
        let instruction_outcome = operation_context.perform_instruction(instruction);

        let mut taken_cycles = instruction.cycles as u64;
        let mut pc_offset = instruction.bytes;

        match instruction_outcome {
            InstructionOutcome::TookConditionalBranch(extra_cycles) => {
                taken_cycles += extra_cycles as u64;
                pc_offset = 0;
            },
            InstructionOutcome::Ok => (),
            InstructionOutcome::ExplicitlySetPC => pc_offset = 0,
        };

        self.increase_pc(pc_offset);

        TCycles(taken_cycles)
    }

    pub fn interrupts_are_enabled(&self) -> bool {
        self.ime
    }

    pub fn enable_interrupts(&mut self) {
        self.ime = true;
    }
    pub fn disable_interrupts(&mut self) {
        self.ime = false;
    }
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        let mut new_flags = z as u8;
        new_flags = (new_flags << 1) | n as u8;
        new_flags = (new_flags << 1) | h as u8;
        new_flags = (new_flags << 1) | c as u8;
        new_flags <<= 4;

        self.set_f(new_flags);
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        (self.get_af() >> flag.get_af_index()) & 0b1 == 1
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        let flag_index = flag.get_af_index();
        let mut f = self.get_f();
        f &= !(0b1 << flag_index);
        f |= (value as u8) << flag_index;
        self.set_f(f);
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
        self.registers[0] = value & 0xFFF0;
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
    pub fn get_sp(&self) -> u16 {
        self.registers[4]
    }
    pub fn set_sp(&mut self, value: u16) {
        self.registers[4] = value;
    }
    pub fn get_pc(&self) -> u16 {
        self.registers[5]
    }
    pub fn increase_pc(&mut self, value: u16) {
        self.registers[5] += value
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

impl Default for Cpu {
    fn default() -> Self {
        Self { registers: Default::default(), ime: Default::default() }
    }
}

pub enum Flag {
    Zero,
    Subtraction,
    HalfCarry,
    Carry,
}

impl Flag {
    fn get_af_index(&self) -> usize {
        match self {
            Flag::Zero => 7,
            Flag::Subtraction => 6,
            Flag::HalfCarry => 5,
            Flag::Carry => 4,
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

pub struct CpuOperationContext<'a, 'b> {
    cpu: &'a mut Cpu,
    bus: &'b mut Bus,
}

impl<'a, 'b> CpuOperationContext<'a, 'b> {
    pub fn new(cpu: &'a mut Cpu, bus: &'b mut Bus) -> Self {
        Self { cpu, bus }
    }

    fn handle_interrupt(&mut self, interrupt: Interrupt) -> TCycles {
        self.bus.lower_interrupt_flag(&interrupt);
        self.cpu.disable_interrupts();

        let isr_address = interrupt.get_isr_address();
        self.push_to_stack(self.cpu.get_pc());
        self.jump(&SixteenBitOperand::Immediate(isr_address));

        TCycles(20)
    }

    fn try_get_interrupt(&self) -> Option<Interrupt> {
        if !self.cpu.ime {
            return None;
        }

        self.bus.try_get_interrupt()
    }

    fn push_to_stack(&mut self, value: u16) {
        let new_sp = self.cpu.get_sp().wrapping_sub(2);
        self.cpu.set_sp(new_sp);
        self.bus.write_u16(new_sp, value)
    }

    fn pop_from_stack(&mut self) -> u16 {
        let sp = self.cpu.get_sp();
        self.cpu.set_sp(sp.wrapping_add(2));
        self.bus.read_u16(sp)
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

    fn get_u8_operand(&mut self, operand: &EightBitOperand) -> u8 {
        match operand {
            EightBitOperand::A => self.cpu.get_a().into(),
            EightBitOperand::B => self.cpu.get_b().into(),
            EightBitOperand::L => self.cpu.get_l().into(),
            EightBitOperand::C => self.cpu.get_c().into(),
            EightBitOperand::D => self.cpu.get_d().into(),
            EightBitOperand::E => self.cpu.get_e().into(),
            EightBitOperand::H => self.cpu.get_h().into(),
            EightBitOperand::HLPointer => self.bus.read(self.cpu.get_hl()),
            EightBitOperand::BCPointer => self.bus.read(self.cpu.get_bc()),
            EightBitOperand::DEPointer => self.bus.read(self.cpu.get_de()),
            EightBitOperand::A16Pointer => {
                let pointer = self.bus.read_u16(self.cpu.get_pc() + 1);
                self.bus.read(pointer)
            },
            EightBitOperand::FF00OffsetByA8 => {
                let offset = self.bus.read(self.cpu.get_pc() + 1);
                self.bus.read(0xFF00 + offset as u16)
            },
            EightBitOperand::FF00OffsetByC => self.bus.read(0xFF00 + self.cpu.get_c() as u16),
            EightBitOperand::N8 => self.bus.read(self.cpu.get_pc() + 1),
            EightBitOperand::Immediate(val) => *val,
            EightBitOperand::HLIPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_add(1));
                self.bus.read(hl)
            },
            EightBitOperand::HLDPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_sub(1));
                self.bus.read(hl)
            },
        }
    }

    fn set_u8_operand(&mut self, operand: &EightBitOperand, value: u8) {
        match operand {
            EightBitOperand::A => {
                self.cpu.set_a(value);
            },
            EightBitOperand::A16Pointer => {
                let pointer = self.bus.read_u16(self.cpu.get_pc() + 1);
                self.bus.write(pointer, value);
            },
            EightBitOperand::B => {
                self.cpu.set_b(value);
            },
            EightBitOperand::HLIPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_add(1));
                self.bus.write(hl, value);
            },
            EightBitOperand::HLDPointer => {
                let hl = self.cpu.get_hl();
                self.cpu.set_hl(hl.wrapping_sub(1));
                self.bus.write(hl, value);
            },
            EightBitOperand::HLPointer => self.bus.write(self.cpu.get_hl(), value),
            EightBitOperand::DEPointer => self.bus.write(self.cpu.get_de(), value),
            EightBitOperand::BCPointer => self.bus.write(self.cpu.get_bc(), value),
            EightBitOperand::L => {
                self.cpu.set_l(value);
            },
            EightBitOperand::C => {
                self.cpu.set_c(value);
            },
            EightBitOperand::D => {
                self.cpu.set_d(value);
            },
            EightBitOperand::E => {
                self.cpu.set_e(value);
            },
            EightBitOperand::FF00OffsetByA8 => {
                let address = self.bus.read(self.cpu.get_pc() + 1);
                self.bus.write(0xFF00 + address as u16, value);
            },
            EightBitOperand::FF00OffsetByC => self.bus.write(0xFF00 + self.cpu.get_c() as u16, value),
            EightBitOperand::H => self.cpu.set_h(value),
            _ => unreachable!("There shouldn't be any places this is called that reaches here."),
        }
    }

    fn get_i8_operand(&mut self) -> i8 {
        self.bus.read(self.cpu.get_pc() + 1) as i8
    }

    fn get_u16_operand(&mut self, operand: &SixteenBitOperand) -> u16 {
        match operand {
            SixteenBitOperand::BC => self.cpu.get_bc().into(),
            SixteenBitOperand::DE => self.cpu.get_de().into(),
            SixteenBitOperand::HL => self.cpu.get_hl().into(),
            SixteenBitOperand::AF => self.cpu.get_af().into(),
            SixteenBitOperand::SP => self.cpu.get_sp().into(),
            SixteenBitOperand::A16 => self.bus.read_u16(self.cpu.get_pc() + 1),
            SixteenBitOperand::N16 => self.bus.read_u16(self.cpu.get_pc() + 1),
            SixteenBitOperand::Immediate(imm) => *imm,
            SixteenBitOperand::E8 => {
                let value = self.bus.read(self.cpu.get_pc() + 1);
                (value as i8) as u16
            },
        }
    }

    fn set_u16_operand(&mut self, operand: &SixteenBitOperand, value: u16) {
        match operand {
            SixteenBitOperand::BC => self.cpu.set_bc(value),
            SixteenBitOperand::DE => self.cpu.set_de(value),
            SixteenBitOperand::HL => self.cpu.set_hl(value),
            SixteenBitOperand::AF => self.cpu.set_af(value),
            SixteenBitOperand::SP => self.cpu.set_sp(value),
            _ => unreachable!("There shouldn't be any places this is called that reaches here."),
        }
    }

    pub fn perform_instruction(&mut self, instruction: &'static Instruction) -> InstructionOutcome {
        match instruction.op_code {
            OpCode::Adc => self.add_with_carry(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Add => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.add_8_bit(&EightBitOperand::try_from(instruction.operands[1]).unwrap())
                },
                OperandType::SixteenBitOperand => self.add_16_bit(
                    &SixteenBitOperand::try_from(instruction.operands[0]).unwrap(),
                    &SixteenBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                _ => unreachable!("There shouldn't be any places this is called that reaches here."),
            },
            OpCode::And => self.and(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Bit => self.bit(
                &&EightBitOperand::try_from(instruction.operands[0]).unwrap(),
                &&EightBitOperand::try_from(instruction.operands[1]).unwrap(),
            ),
            OpCode::Call => match instruction.operands.len() {
                1 => self.call(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap()),
                2 => self.call_conditional(
                    &Condition::try_from(instruction.operands[0]).unwrap(),
                    &SixteenBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                _ => unreachable!("OpCode only can have 1 or 2 operands"),
            },
            OpCode::Ccf => self.complement_carry_flag(),
            OpCode::Cp => self.compare(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Cpl => self.complement(),
            OpCode::Daa => self.decimal_adjust_accumulator(),
            OpCode::Dec => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.decrement_8_bit(&EightBitOperand::try_from(instruction.operands[0]).unwrap())
                },
                OperandType::SixteenBitOperand => {
                    self.decrement_16_bit(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap())
                },
                _ => unreachable!("There shouldn't be any places this is called that reaches here."),
            },
            OpCode::Di => self.disable_interrupts(),
            OpCode::Ei => self.enable_interrupts(),
            OpCode::Halt => self.halt(),
            OpCode::Illegal => unreachable!("We should have caught this before here."),
            OpCode::Inc => match OperandType::from(instruction.operands[0]) {
                OperandType::EightBitOperand => {
                    self.increment_8_bit(&EightBitOperand::try_from(instruction.operands[0]).unwrap())
                },
                OperandType::SixteenBitOperand => {
                    self.increment_16_bit(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap())
                },
                _ => unreachable!("There shouldn't be any places this is called that reaches here."),
            },
            OpCode::Jp => match instruction.operands.len() {
                1 => self.jump(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap()),
                2 => self.jump_conditional(
                    &Condition::try_from(instruction.operands[0]).unwrap(),
                    &SixteenBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                _ => unreachable!("OpCode only can have 1 or 2 operands"),
            },
            OpCode::Jr => match instruction.operands.len() {
                1 => self.jump_relative(&SignedEightBitOperand::try_from(instruction.operands[0]).unwrap()),
                2 => self.jump_relative_conditional(
                    &Condition::try_from(instruction.operands[0]).unwrap(),
                    &SignedEightBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                _ => unreachable!("OpCode only can have 1 or 2 operands"),
            },

            OpCode::Ld => match (
                OperandType::from(instruction.operands[0]),
                OperandType::from(instruction.operands[1]),
            ) {
                (OperandType::EightBitOperand, OperandType::EightBitOperand) => self.load_8_bit(
                    &EightBitOperand::try_from(instruction.operands[0]).unwrap(),
                    &EightBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                (OperandType::SixteenBitOperand, OperandType::SixteenBitOperand) => self.load_16_bit(
                    &SixteenBitOperand::try_from(instruction.operands[0]).unwrap(),
                    &SixteenBitOperand::try_from(instruction.operands[1]).unwrap(),
                ),
                // Special case for LD [A16] SP
                (OperandType::EightBitOperand, OperandType::SixteenBitOperand) => self.load_a16_pointer_sp(),
                _ => unreachable!("There shouldn't be any places this is called that reaches here."),
            },
            OpCode::Ldh => self.load_high(
                &EightBitOperand::try_from(instruction.operands[0]).unwrap(),
                &EightBitOperand::try_from(instruction.operands[1]).unwrap(),
            ),
            OpCode::Nop => self.nop(),
            OpCode::Or => self.or(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Pop => self.pop(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Prefix => unreachable!("This should have been caught prior"),
            OpCode::Push => self.push(&SixteenBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Res => self.clear_bit(
                &EightBitOperand::try_from(instruction.operands[0]).unwrap(),
                &EightBitOperand::try_from(instruction.operands[1]).unwrap(),
            ),
            OpCode::Ret => match instruction.operands.len() {
                0 => self.ret(),
                1 => self.return_conditional(&Condition::try_from(instruction.operands[0]).unwrap()),
                _ => unreachable!("OpCode only can have 0 or 1 operands"),
            },
            OpCode::Reti => self.return_from_interrupt(),
            OpCode::Rl => self.rotate_left_through_carry(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Rla => self.rotate_left_through_carry_a(),
            OpCode::Rlc => self.rotate_left_into_carry(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Rlca => self.rotate_left_into_carry_a(),
            OpCode::Rr => self.rotate_right_through_carry(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Rra => self.rotate_right_through_carry_a(),
            OpCode::Rrc => self.rotate_right_into_carry(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Rrca => self.rotate_right_into_carry_a(),
            OpCode::Rst => self.call_vector(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Sbc => self.subtract_with_carry(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Scf => self.set_carry_flag(),
            OpCode::Set => self.set_bit(
                &EightBitOperand::try_from(instruction.operands[0]).unwrap(),
                &EightBitOperand::try_from(instruction.operands[1]).unwrap(),
            ),
            OpCode::Sla => self.shift_left_arithmetically(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Sra => {
                self.shift_right_arithmetically(&EightBitOperand::try_from(instruction.operands[0]).unwrap())
            },
            OpCode::Srl => self.shift_right_logically(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Stop => self.stop(),
            OpCode::Sub => self.subtract(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
            OpCode::Swap => self.swap(&EightBitOperand::try_from(instruction.operands[0]).unwrap()),
            OpCode::Xor => self.xor(&EightBitOperand::try_from(instruction.operands[1]).unwrap()),
        }
    }

    fn add_with_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand);
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

        InstructionOutcome::Ok
    }

    fn add_8_bit(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand);

        let (result, overflowed) = a.overflowing_add(operand_value);

        self.set_a(result);
        self.set_flags(result == 0, false, bit_3_overflow(vec![a, operand_value]), overflowed);

        InstructionOutcome::Ok
    }

    fn add_16_bit(&mut self, operand0: &SixteenBitOperand, operand1: &SixteenBitOperand) -> InstructionOutcome {
        let operand0_value = self.get_u16_operand(operand0);
        let operand1_value = self.get_u16_operand(operand1);

        let (result, overflowed) = operand0_value.overflowing_add(operand1_value);

        self.set_u16_operand(operand0, result);

        self.set_flags(
            self.cpu.get_flag(Flag::Zero),
            false,
            bit_11_overflow(vec![operand0_value, operand1_value]),
            overflowed,
        );

        InstructionOutcome::Ok
    }

    fn and(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(operand);
        let a = self.get_a();

        let result = operand_value & a;

        self.set_a(result);
        self.set_flags(result == 0, false, true, false);

        InstructionOutcome::Ok
    }

    fn bit(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionOutcome {
        let test_bit = self.get_u8_operand(operand0);
        let byte = self.get_u8_operand(operand1);

        let result = (byte >> test_bit) & 0b1;

        self.set_flags(result == 0, false, true, self.cpu.get_flag(Flag::Carry));

        InstructionOutcome::Ok
    }

    fn call(&mut self, address: &SixteenBitOperand) -> InstructionOutcome {
        self.push_to_stack(self.cpu.get_pc() + 3);
        self.jump(address);

        InstructionOutcome::ExplicitlySetPC
    }

    fn call_conditional(&mut self, condition: &Condition, address: &SixteenBitOperand) -> InstructionOutcome {
        if self.check_condition(condition) {
            self.call(address);
            InstructionOutcome::TookConditionalBranch(12)
        } else {
            InstructionOutcome::Ok
        }
    }

    fn complement_carry_flag(&mut self) -> InstructionOutcome {
        let carry = self.cpu.get_flag(Flag::Carry);
        self.cpu.set_flag(Flag::Carry, !carry);

        InstructionOutcome::Ok
    }

    fn compare(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(&operand);
        let a = self.get_a();
        self.set_flags(
            a == operand_value,
            true,
            bit_4_borrow(a, operand_value, false),
            a < operand_value,
        );

        InstructionOutcome::Ok
    }

    fn complement(&mut self) -> InstructionOutcome {
        let a = self.get_a();
        self.set_a(!a);

        self.set_flags(
            self.cpu.get_flag(Flag::Zero),
            true,
            true,
            self.cpu.get_flag(Flag::Carry),
        );

        InstructionOutcome::Ok
    }

    fn decimal_adjust_accumulator(&mut self) -> InstructionOutcome {
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

    fn decrement_8_bit(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(operand);

        let result = operand_value.wrapping_sub(1);

        self.set_u8_operand(operand, result);
        self.set_flags(
            result == 0,
            true,
            bit_4_borrow(operand_value, 1, false),
            self.cpu.get_flag(Flag::Carry),
        );

        InstructionOutcome::Ok
    }

    fn decrement_16_bit(&mut self, operand: &SixteenBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u16_operand(operand);

        let result = operand_value.wrapping_sub(1);

        self.set_u16_operand(operand, result);

        InstructionOutcome::Ok
    }

    fn increment_8_bit(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(operand);

        let result = operand_value.wrapping_add(1);

        self.set_u8_operand(operand, result);
        self.set_flags(
            result == 0,
            false,
            bit_3_overflow(vec![operand_value, 1]),
            self.cpu.get_flag(Flag::Carry),
        );

        InstructionOutcome::Ok
    }

    fn increment_16_bit(&mut self, operand: &SixteenBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u16_operand(operand);

        let result = operand_value.wrapping_add(1);

        self.set_u16_operand(operand, result);

        InstructionOutcome::Ok
    }

    fn jump(&mut self, operand: &SixteenBitOperand) -> InstructionOutcome {
        let address = self.get_u16_operand(operand);
        self.cpu.set_pc(address);

        InstructionOutcome::ExplicitlySetPC
    }

    fn jump_conditional(&mut self, condition: &Condition, address: &SixteenBitOperand) -> InstructionOutcome {
        if self.check_condition(condition) {
            self.jump(address);
            InstructionOutcome::TookConditionalBranch(4)
        } else {
            InstructionOutcome::Ok
        }
    }

    fn jump_relative(&mut self, _operand: &SignedEightBitOperand) -> InstructionOutcome {
        let offset = self.get_i8_operand();
        let jump_address = offset as i16 + self.cpu.get_pc() as i16 + 2;

        self.cpu.set_pc(jump_address as u16);

        InstructionOutcome::ExplicitlySetPC
    }

    fn jump_relative_conditional(
        &mut self,
        condition: &Condition,
        operand: &SignedEightBitOperand,
    ) -> InstructionOutcome {
        if self.check_condition(condition) {
            self.jump_relative(operand);
            InstructionOutcome::TookConditionalBranch(4)
        } else {
            InstructionOutcome::Ok
        }
    }

    fn load_8_bit(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand1);
        self.set_u8_operand(operand0, value);

        InstructionOutcome::Ok
    }

    fn load_16_bit(&mut self, operand0: &SixteenBitOperand, operand1: &SixteenBitOperand) -> InstructionOutcome {
        let value = self.get_u16_operand(operand1);
        self.set_u16_operand(operand0, value);

        InstructionOutcome::Ok
    }

    fn load_a16_pointer_sp(&mut self) -> InstructionOutcome {
        let address = self.get_u16_operand(&SixteenBitOperand::A16);
        let sp = self.get_u16_operand(&SixteenBitOperand::SP);
        self.bus.write_u16(address, sp);

        InstructionOutcome::Ok
    }

    fn load_high(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand1);
        self.set_u8_operand(operand0, value);

        InstructionOutcome::Ok
    }

    fn or(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);
        let a = self.get_a();

        let result = value | a;

        self.set_a(result);
        self.cpu.set_flags(result == 0, false, false, false);

        InstructionOutcome::Ok
    }

    fn pop(&mut self, operand: &SixteenBitOperand) -> InstructionOutcome {
        let value = self.pop_from_stack();

        self.set_u16_operand(operand, value);

        InstructionOutcome::Ok
    }

    fn push(&mut self, operand: &SixteenBitOperand) -> InstructionOutcome {
        let value = self.get_u16_operand(operand);

        self.push_to_stack(value);

        InstructionOutcome::Ok
    }

    fn clear_bit(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionOutcome {
        let bit_number = self.get_u8_operand(operand0);
        let byte = self.get_u8_operand(operand1);

        let mask = !(1 << bit_number);
        let result = byte & mask;

        self.set_u8_operand(operand1, result);

        InstructionOutcome::Ok
    }

    fn ret(&mut self) -> InstructionOutcome {
        let new_pc = self.pop_from_stack();
        self.cpu.set_pc(new_pc);

        InstructionOutcome::ExplicitlySetPC
    }

    fn return_conditional(&mut self, condition: &Condition) -> InstructionOutcome {
        if self.check_condition(condition) {
            self.ret();
            InstructionOutcome::TookConditionalBranch(12)
        } else {
            InstructionOutcome::Ok
        }
    }

    fn return_from_interrupt(&mut self) -> InstructionOutcome {
        self.enable_interrupts();
        self.ret()
    }

    fn rotate_left_through_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);
        let carry = self.cpu.get_flag(Flag::Carry);

        let new_carry = (value >> 7) == 1;
        let result = (value << 1) | carry as u8;

        self.set_u8_operand(operand, result);

        self.cpu.set_flags(result == 0, false, false, new_carry);

        InstructionOutcome::Ok
    }

    fn rotate_left_through_carry_a(&mut self) -> InstructionOutcome {
        self.rotate_left_through_carry(&EightBitOperand::A);

        self.cpu.set_flag(Flag::Zero, false);

        InstructionOutcome::Ok
    }

    fn rotate_left_into_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);

        let new_carry = value >> 7;
        let result = (value << 1) | new_carry;

        self.set_u8_operand(operand, result);

        self.cpu.set_flags(result == 0, false, false, new_carry == 1);

        InstructionOutcome::Ok
    }

    fn rotate_left_into_carry_a(&mut self) -> InstructionOutcome {
        self.rotate_left_into_carry(&EightBitOperand::A);

        self.cpu.set_flag(Flag::Zero, false);

        InstructionOutcome::Ok
    }

    fn rotate_right_through_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);
        let carry = (self.cpu.get_flag(Flag::Carry) as u8) << 7;

        let new_carry = (value & 1) == 1;
        let result = (value >> 1) | carry;

        self.set_u8_operand(operand, result);

        self.cpu.set_flags(result == 0, false, false, new_carry);

        InstructionOutcome::Ok
    }

    fn rotate_right_through_carry_a(&mut self) -> InstructionOutcome {
        self.rotate_right_through_carry(&EightBitOperand::A);

        self.cpu.set_flag(Flag::Zero, false);

        InstructionOutcome::Ok
    }

    fn rotate_right_into_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);

        let new_carry = value << 7;
        let result = (value >> 1) | new_carry;

        self.set_u8_operand(operand, result);

        self.cpu.set_flags(result == 0, false, false, new_carry != 0);

        InstructionOutcome::Ok
    }

    fn rotate_right_into_carry_a(&mut self) -> InstructionOutcome {
        self.rotate_right_into_carry(&EightBitOperand::A);

        self.cpu.set_flag(Flag::Zero, false);

        InstructionOutcome::Ok
    }

    fn call_vector(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);
        self.call(&SixteenBitOperand::Immediate(value as u16));

        InstructionOutcome::Ok
    }

    fn subtract_with_carry(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand);
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

        InstructionOutcome::Ok
    }

    fn set_carry_flag(&mut self) -> InstructionOutcome {
        self.cpu.set_flag(Flag::Carry, true);

        self.set_flags(self.cpu.get_flag(Flag::Zero), false, false, true);

        InstructionOutcome::Ok
    }

    fn set_bit(&mut self, operand0: &EightBitOperand, operand1: &EightBitOperand) -> InstructionOutcome {
        let bit_number = self.get_u8_operand(operand0);
        let byte = self.get_u8_operand(operand1);

        let mask = 1 << bit_number;
        let result = byte | mask;

        self.set_u8_operand(operand1, result);

        InstructionOutcome::Ok
    }

    fn shift_left_arithmetically(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);

        let new_carry = value >> 7 == 1;
        let result = value << 1;

        self.set_u8_operand(operand, result);
        self.cpu.set_flags(result == 0, false, false, new_carry);

        InstructionOutcome::Ok
    }

    fn shift_right_arithmetically(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);
        let sign = value >> 7;

        let new_carry = value & 1 == 1;
        let result = (value >> 1) | sign;

        self.set_u8_operand(operand, result);
        self.cpu.set_flags(result == 0, false, false, new_carry);

        InstructionOutcome::Ok
    }

    fn shift_right_logically(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let value = self.get_u8_operand(operand);

        let new_carry = value & 1 == 1;
        let result = value >> 1;

        self.set_u8_operand(operand, result);
        self.cpu.set_flags(result == 0, false, false, new_carry);

        InstructionOutcome::Ok
    }

    fn subtract(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let a = self.get_a();
        let operand_value = self.get_u8_operand(operand);

        let (result, borrowed) = a.overflowing_sub(operand_value);

        self.set_a(result);
        self.set_flags(result == 0, true, bit_4_borrow(a, operand_value, false), borrowed);

        InstructionOutcome::Ok
    }

    fn swap(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(operand);

        let result = operand_value.rotate_left(4);

        self.set_u8_operand(operand, result);
        self.set_flags(result == 0, false, false, false);

        InstructionOutcome::Ok
    }

    fn xor(&mut self, operand: &EightBitOperand) -> InstructionOutcome {
        let operand_value = self.get_u8_operand(operand);
        let a = self.get_a();

        let result = a ^ operand_value;

        self.set_a(result);
        self.cpu.set_flags(result == 0, false, false, false);

        InstructionOutcome::Ok
    }

    fn disable_interrupts(&mut self) -> InstructionOutcome {
        self.cpu.disable_interrupts();
        InstructionOutcome::Ok
    }

    // Enabling interrupts has a one cycle delay so we'll simulate that by raising an event that
    // is handled by raising ANOTHER event that is handled by setting ime to true.
    // A little wonky, but I think it's better than having a "should_set_ime" flag that we check every cycle
    fn enable_interrupts(&mut self) -> InstructionOutcome {
        notate_event(GameBoyEvent::IeTriggered);
        InstructionOutcome::Ok
    }

    fn halt(&mut self) -> InstructionOutcome {
        notate_event(GameBoyEvent::ChangeGameBoyMode(GameBoyMode::Halted));
        InstructionOutcome::Ok
    }

    fn nop(&self) -> InstructionOutcome {
        InstructionOutcome::Ok
    }

    fn stop(&mut self) -> InstructionOutcome {
        notate_event(GameBoyEvent::ChangeGameBoyMode(GameBoyMode::Stopped));
        InstructionOutcome::Ok
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

fn bit_11_overflow(operands: Vec<u16>) -> bool {
    let mut result = 0;
    operands.iter().for_each(|o| result += o & 0x0FFF);
    result > 0x0FFF
}

#[cfg(test)]
mod test {
    use crate::processor::instructions::Instruction;

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
