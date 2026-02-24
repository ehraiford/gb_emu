use crate::{
    bus::Bus,
    game_boy::{GameBoyEvent, notate_event},
    io_devices::interrupts::Interrupt,
    processor::{
        instruction_tables::{CBPREFIXED, UNPREFIXED},
        instructions::{Condition, Instruction, OpCode, Operand, m_cycle_accuracy::MicroOp},
    },
};

#[derive(Default)]
pub struct Cpu {
    af: u16,
    bc: u16,
    de: u16,
    sp: u16,
    hl: u16,
    pc: u16,

    ime: bool,
    ei_delay: u8, // special field to add cycle delay to enabling interrupts
    state: CpuState,
    instruction_state_machine: InstructionStateMachine,
}

impl Cpu {
    pub fn tick(&mut self, bus: &mut Bus) {
        let mut cpu_operation_context = CpuOperationContext::new(self, bus);
        cpu_operation_context.tick();
    }

    pub fn interrupts_are_enabled(&self) -> bool {
        self.ime
    }

    fn get_a(&self) -> u8 {
        (self.af >> 8) as u8
    }

    pub fn set_a(&mut self, new_a: u8) {
        let f = self.get_f() as u16;
        let a = (new_a as u16) << 8;

        self.af = a | f;
    }
    pub fn set_b(&mut self, new_b: u8) {
        let c = self.get_c() as u16;
        let b = (new_b as u16) << 8;

        self.bc = b | c;
    }
    pub fn set_d(&mut self, new_d: u8) {
        let e = self.get_e() as u16;
        let d = (new_d as u16) << 8;

        self.de = d | e;
    }
    pub fn set_h(&mut self, new_h: u8) {
        let l = self.get_l() as u16;
        let h = (new_h as u16) << 8;

        self.hl = h | l;
    }
    fn get_f(&self) -> u8 {
        (self.af & 0xFF) as u8
    }
    fn set_f(&mut self, new_f: u8) {
        self.af = (self.af & 0xFF00) | (new_f as u16) & 0xF0
    }
    fn get_c(&self) -> u8 {
        (self.bc & 0xFF) as u8
    }
    fn set_c(&mut self, new_c: u8) {
        self.bc = (self.bc & 0xFF00) | new_c as u16
    }
    fn get_e(&self) -> u8 {
        (self.de & 0xFF) as u8
    }
    fn set_e(&mut self, new_e: u8) {
        self.de = (self.de & 0xFF00) | new_e as u16
    }
    fn set_l(&mut self, new_l: u8) {
        self.hl = (self.hl & 0xFF00) | new_l as u16
    }
    fn get_l(&self) -> u8 {
        (self.hl & 0xFF) as u8
    }

    fn disable_interrupts(&mut self) {
        self.ime = false;
    }
    pub fn enable_interrupts(&mut self) {
        self.ei_delay = 2; // one for EI, one for the next instruction
    }

    fn get_flag(&self, flag: Flag) -> bool {
        (self.af >> flag.af_index()) & 0b1 == 1
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        let flag_index = flag.af_index();
        let mut f = self.get_f();
        f &= !(0b1 << flag_index);
        f |= (value as u8) << flag_index;
        self.set_f(f);
    }

    fn check_condition(&self, cond: &Condition) -> bool {
        match cond {
            Condition::NotZero => !self.get_flag(Flag::Zero),
            Condition::Zero => self.get_flag(Flag::Zero),
            Condition::NotCarry => !self.get_flag(Flag::Carry),
            Condition::Carry => self.get_flag(Flag::Carry),
        }
    }

    fn update_to_next_instruction(&mut self, instruction: &'static Instruction) {
        self.instruction_state_machine.update_to_next_instruction(instruction);
        self.state = CpuState::StartingNewInstruction;
    }

    /// Performs the current instruction's logic. Think of it as an "ALU+"
    /// The "+" being that it also handles
    ///     1. Enabling and disabling interrupts
    ///     2. Writing results back to registers when appropriate
    ///     3. Updating flag values
    /// Basically, anything not expressed through the other micro ops is done here.
    fn perform_instruction_logic(&mut self, operands: CalcedOperands) {
        match self.instruction_state_machine.instruction.op_code {
            OpCode::Adc => self.add_with_carry(operands.get_u8s().unwrap()),
            OpCode::Add => self.add(operands),
            OpCode::And => self.and(operands.get_u8s().unwrap()),
            OpCode::Bit => self.bit(operands.get_u8s().unwrap()),
            OpCode::Call => self.call(operands.get_cond()),
            OpCode::Ccf => self.complement_carry_flag(),
            OpCode::Cp => self.compare(operands.get_u8s().unwrap()),
            OpCode::Cpl => self.complement(),
            OpCode::Di => self.disable_interrupts(),
            OpCode::Ei => self.enable_interrupts(),
            OpCode::Daa => self.decimal_adjust_accumulator(),
            OpCode::Dec => self.decrement(operands),
            OpCode::Halt => self.halt(operands.get_cond().unwrap()),
            OpCode::Jp => self.jump(operands),
            OpCode::Inc => self.increment(operands),
            OpCode::Jr => self.jump_relative(operands),
            OpCode::Ld => self.load(operands),
            OpCode::Ldh => self.load_high(operands.get_u8s().unwrap()),
            OpCode::Or => self.or(operands.get_u8s().unwrap()),
            OpCode::Pop => self.pop(operands.get_u16().unwrap()),
            OpCode::Res => self.clear_bit(operands.get_u8s().unwrap()),
            OpCode::Ret => self.ret(operands.get_cond()),
            OpCode::Reti => self.reti(),
            OpCode::Rl => self.rotate_left_through_carry(operands.get_u8().unwrap()),
            OpCode::Rla => self.rotate_left_through_carry_a(),
            OpCode::Rlc => self.rotate_left_into_carry(operands.get_u8().unwrap()),
            OpCode::Rlca => self.rotate_left_into_carry_a(),
            OpCode::Rr => self.rotate_right_through_carry(operands.get_u8().unwrap()),
            OpCode::Rra => self.rotate_right_through_carry_a(),
            OpCode::Rrc => self.rotate_right_into_carry(operands.get_u8().unwrap()),
            OpCode::Rrca => self.rotate_right_into_carry_a(),
            OpCode::Sbc => self.subtract_with_carry(operands.get_u8s().unwrap()),
            OpCode::Scf => self.set_carry_flag(),
            OpCode::Set => self.set_bit(operands.get_u8s().unwrap()),
            OpCode::Sla => self.shift_left_arithmetically(operands.get_u8().unwrap()),
            OpCode::Sra => self.shift_right_arithmetically(operands.get_u8().unwrap()),
            OpCode::Srl => self.shift_right_logically(operands.get_u8().unwrap()),
            OpCode::Stop => self.stop(),
            OpCode::Sub => self.subtract(operands.get_u8s().unwrap()),
            OpCode::Swap => self.swap(operands.get_u8().unwrap()),
            OpCode::Xor => self.xor(operands.get_u8s().unwrap()),

            // all of the logic for the following instructions happen automatically in other steps
            OpCode::Nop | OpCode::Rst | OpCode::Push | OpCode::Prefix => (),

            OpCode::Illegal => notate_event(GameBoyEvent::TriedRunningIllegalInstruction),
        }
    }

    fn decode_instruction(&mut self) {
        for i in 0..2 {
            if let Some(operand) = self.instruction_state_machine.instruction.operands.get(i as usize) {
                let operand_value = self.decode_operand(operand);
                self.set_instruction_operand(operand_value, i);
            } else {
                self.set_instruction_operand(OperandValue::Unused, i);
            }
        }
    }

    fn get_eight_bit_register(&self, register: EightBitRegister) -> OperandValue {
        let value = match register {
            EightBitRegister::A => (self.af >> 8) as u8,
            EightBitRegister::B => (self.bc >> 8) as u8,
            EightBitRegister::C => (self.bc & 0xFF) as u8,
            EightBitRegister::D => (self.de >> 8) as u8,
            EightBitRegister::E => (self.de & 0xFF) as u8,
            EightBitRegister::H => (self.hl >> 8) as u8,
            EightBitRegister::L => (self.hl & 0xFF) as u8,
        };

        OperandValue::U8(U8Operand::Calculated(value))
    }

    fn get_sixteen_bit_register(&mut self, register: SixteenBitRegister) -> OperandValue {
        let value = match register {
            SixteenBitRegister::AF => self.af,
            SixteenBitRegister::BC => self.bc,
            SixteenBitRegister::DE => self.de,
            SixteenBitRegister::SP => self.sp,
            SixteenBitRegister::HL => self.hl,
        };
        OperandValue::U16(U16Operand::Calculated(value))
    }

    fn get_register_as_pointer(&mut self, register: SixteenBitRegister) -> OperandValue {
        let OperandValue::U16(U16Operand::Calculated(value)) = self.get_sixteen_bit_register(register) else {
            unreachable!("There is no operation that will have the state machine call this and fail")
        };
        OperandValue::Pointer(PointerOperand::Calculated(value))
    }

    fn decode_operand(&mut self, operand: &Operand) -> OperandValue {
        match operand {
            Operand::Immediate(imm) => OperandValue::U8(U8Operand::Calculated(*imm)),
            Operand::A => self.get_eight_bit_register(EightBitRegister::A),
            Operand::B => self.get_eight_bit_register(EightBitRegister::B),
            Operand::C => self.get_eight_bit_register(EightBitRegister::C),
            Operand::D => self.get_eight_bit_register(EightBitRegister::D),
            Operand::E => self.get_eight_bit_register(EightBitRegister::E),
            Operand::H => self.get_eight_bit_register(EightBitRegister::H),
            Operand::L => self.get_eight_bit_register(EightBitRegister::L),
            Operand::AF => self.get_sixteen_bit_register(SixteenBitRegister::AF),
            Operand::BC => self.get_sixteen_bit_register(SixteenBitRegister::BC),
            Operand::DE => self.get_sixteen_bit_register(SixteenBitRegister::DE),
            Operand::HL => self.get_sixteen_bit_register(SixteenBitRegister::HL),
            Operand::SP => self.get_sixteen_bit_register(SixteenBitRegister::SP),
            Operand::BCPointer => self.get_register_as_pointer(SixteenBitRegister::BC),
            Operand::DEPointer => self.get_register_as_pointer(SixteenBitRegister::DE),
            Operand::FF00OffsetByC => OperandValue::Pointer(PointerOperand::Calculated(0xFF00 + (self.bc & 0x00FF))),
            Operand::HLPointer => self.get_register_as_pointer(SixteenBitRegister::HL),
            Operand::HLDPointer => OperandValue::Pointer(PointerOperand::HLD(self.hl)),
            Operand::HLIPointer => OperandValue::Pointer(PointerOperand::HLI(self.hl)),
            Operand::Carry => OperandValue::Condition(self.check_condition(&Condition::Carry)),
            Operand::NotCarry => OperandValue::Condition(self.check_condition(&Condition::NotCarry)),
            Operand::NotZero => OperandValue::Condition(self.check_condition(&Condition::NotZero)),
            Operand::Zero => OperandValue::Condition(self.check_condition(&Condition::Zero)),
            Operand::A16 | Operand::N16 => OperandValue::U16(U16Operand::NotYetCalculated),
            Operand::A16Pointer => OperandValue::Pointer(PointerOperand::NotYetCalculated),
            Operand::E8 => OperandValue::I8(I8Operand::NotCalculated),
            Operand::FF00OffsetByA8 => OperandValue::Pointer(PointerOperand::CalculatedMsb(0xFF)),
            Operand::N8 => OperandValue::U8(U8Operand::NotCalculated),
        }
    }

    /// Sets the value for an operand in the instruction state machine.
    /// If both operands have been calculated, this performs the instruction's "logic", too
    fn set_instruction_operand(&mut self, value: OperandValue, operand_num: u8) {
        self.instruction_state_machine.set_operand(value, operand_num);
        if let Some(operands) = self.instruction_state_machine.get_calced_operands() {
            self.perform_instruction_logic(operands);
        }
    }

    fn set_operand_lsb(&mut self, lsb: u8, operand_num: u8) {
        self.instruction_state_machine.set_operand_lsb(lsb, operand_num);
        if let Some(operands) = self.instruction_state_machine.get_calced_operands() {
            self.perform_instruction_logic(operands);
        }
    }
    fn set_operand_msb(&mut self, msb: u8, operand_num: u8) {
        self.instruction_state_machine.set_operand_msb(msb, operand_num);
        if let Some(operands) = self.instruction_state_machine.get_calced_operands() {
            self.perform_instruction_logic(operands);
        }
    }

    /// Sets the result in the instruction state machine.
    /// If the operand that will eventually store the result is a register, we go ahead and write it back, too
    fn set_instruction_result(&mut self, value: u16, destination_operand_num: u8) {
        let state_machine = &mut self.instruction_state_machine;
        state_machine.result = value;
        match state_machine.instruction.operands[destination_operand_num as usize] {
            Operand::A => self.set_a(value as u8),
            Operand::B => self.set_b(value as u8),
            Operand::C => self.set_c(value as u8),
            Operand::D => self.set_d(value as u8),
            Operand::E => self.set_e(value as u8),
            Operand::H => self.set_h(value as u8),
            Operand::L => self.set_l(value as u8),
            Operand::AF => self.af = value & 0xFFF0, // lower nibble of f is always 0
            Operand::BC => self.bc = value,
            Operand::DE => self.de = value,
            Operand::HL => self.hl = value,
            Operand::SP => self.sp = value,
            _ => (),
        }
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        let mut new_flags = z as u8;
        new_flags = (new_flags << 1) | n as u8;
        new_flags = (new_flags << 1) | h as u8;
        new_flags = (new_flags << 1) | c as u8;
        new_flags <<= 4;

        self.set_f(new_flags);
    }
}

// Instruction Logic Methods
impl Cpu {
    fn add(&mut self, operands: CalcedOperands) {
        match operands {
            CalcedOperands::TwoU8(operand_0, operand_1) => self.add_8_bit(operand_0, operand_1),
            CalcedOperands::TwoU16(operand_0, operand_1) => self.add_16_bit(operand_0, operand_1),
            CalcedOperands::AddSpPlusE8(sp, e8) => self.add_sp_e8(sp, e8),
            _ => unreachable!("There won't be cases with fewer than 2 operands"),
        }
    }
    fn add_8_bit(&mut self, operand_0: u8, operand_1: u8) {
        let (result, overflowed) = operand_0.overflowing_add(operand_1);

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, bit_3_overflow(operand_0, operand_1), overflowed);
    }
    fn add_16_bit(&mut self, operand_0: u16, operand_1: u16) {
        let (result, overflowed) = operand_0.overflowing_add(operand_1);

        self.set_instruction_result(result, 0);
        self.set_flags(
            self.get_flag(Flag::Zero),
            false,
            (operand_0 & 0x0FFF) + (operand_1 & 0x0FFF) > 0xFFF,
            overflowed,
        );
    }
    fn add_sp_e8(&mut self, sp: u16, e8: i8) {
        let result = sp.wrapping_add(e8 as i16 as u16);

        self.set_instruction_result(result, 0);
        self.set_flags(
            false,
            false,
            bit_3_overflow(sp as u8, e8 as u8),
            bit_7_overflow(sp as u8, e8 as u8),
        );
    }

    fn add_with_carry(&mut self, operands: (u8, u8)) {
        let carry = self.get_flag(Flag::Carry);

        let (middle_result, overflowed_middle) = operands.0.overflowing_add(operands.1);
        let (result, overflowed_end) = middle_result.overflowing_add(carry as u8);

        self.set_instruction_result(result as u16, 0);
        self.set_flags(
            result == 0,
            false,
            ((carry as u8 & 0x0F) + (operands.0 & 0x0F) + (operands.1 & 0x0F)) > 0x0F,
            overflowed_middle | overflowed_end,
        );
    }

    fn and(&mut self, operands: (u8, u8)) {
        let result = operands.0 & operands.1;
        self.set_flags(result == 0, false, true, false);
        self.set_instruction_result(result as u16, 0);
    }

    fn bit(&mut self, operands: (u8, u8)) {
        let result = (operands.1 >> operands.0) & 0b1;
        self.set_flags(result == 0, false, true, self.get_flag(Flag::Carry));
    }

    /// Just checks that the condition has been met (if there is one).
    /// If it has not been, it ends the instruction early.
    /// The actual logic for Call is handled across other M Cycles.
    /// It DOES NOT happen here.
    fn call(&mut self, condition_met: Option<bool>) {
        if !condition_met.unwrap_or(true) {
            self.instruction_state_machine.end_instruction_early();
        }
    }

    fn complement_carry_flag(&mut self) {
        let new_carry = !self.get_flag(Flag::Carry);

        self.set_flags(self.get_flag(Flag::Zero), false, false, new_carry);
    }

    fn compare(&mut self, operands: (u8, u8)) {
        self.set_flags(
            operands.0 == operands.1,
            true,
            bit_4_borrow(operands.0, operands.1, false),
            operands.0 < operands.1,
        );
    }

    fn complement(&mut self) {
        // complement is only done on `a` so we'll just skip passing in the operands here and grab a ourselves
        let a = self.get_a();
        self.set_a(!a);
        self.set_flags(self.get_flag(Flag::Zero), true, true, self.get_flag(Flag::Carry));
    }

    fn decimal_adjust_accumulator(&mut self) {
        let mut a = self.get_a();
        let mut adjustment = 0;
        let mut new_carry = false;

        if self.get_flag(Flag::Subtraction) {
            if self.get_flag(Flag::HalfCarry) {
                adjustment |= 0x06;
            }
            if self.get_flag(Flag::Carry) {
                adjustment |= 0x60;
                new_carry = true;
            }
        } else {
            if (a & 0x0F) > 0x09 || self.get_flag(Flag::HalfCarry) {
                adjustment |= 0x06;
            }
            if a > 0x99 || self.get_flag(Flag::Carry) {
                adjustment |= 0x60;
                new_carry = true;
            }
        }

        if self.get_flag(Flag::Subtraction) {
            a = a.wrapping_sub(adjustment);
        } else {
            a = a.wrapping_add(adjustment);
        }

        self.set_a(a);
        self.set_flags(
            a == 0,
            self.get_flag(Flag::Subtraction),
            false,
            new_carry || self.get_flag(Flag::Carry),
        );
    }

    fn decrement(&mut self, operands: CalcedOperands) {
        match operands {
            CalcedOperands::OneU8(operand) => self.decrement_8_bit(operand),
            CalcedOperands::OneU16(operand) => self.decrement_16_bit(operand),
            _ => unreachable!("No other case will get here"),
        }
    }
    fn decrement_16_bit(&mut self, operand: u16) {
        let result = operand.wrapping_sub(1);
        self.set_instruction_result(result, 0);
    }
    fn decrement_8_bit(&mut self, operand: u8) {
        let result = operand.wrapping_sub(1);
        self.set_instruction_result(result as u16, 0);
        self.set_flags(
            result == 0,
            true,
            bit_4_borrow(operand, 1, false),
            self.get_flag(Flag::Carry),
        );
    }

    fn halt(&mut self, pending_interrupts: bool) {
        // if IME is disabled but we have pending interrupts, we do the halt bug
        match !self.ime && pending_interrupts && self.ei_delay == 0 {
            true => self.state = CpuState::HaltBug,
            false => self.state = CpuState::Halted,
        };
        notate_event(GameBoyEvent::ChangeGameBoyMode(crate::game_boy::GameBoyMode::Halted));
    }

    fn jump(&mut self, operands: CalcedOperands) {
        let new_pc = match operands {
            CalcedOperands::OneU16(address) => address,
            CalcedOperands::CondU16(condition_met, address) => match condition_met {
                true => address,
                false => return self.instruction_state_machine.end_instruction_early(),
            },
            _ => unreachable!("No other case will get here"),
        };
        self.pc = new_pc;
    }

    fn increment(&mut self, operands: CalcedOperands) {
        match operands {
            CalcedOperands::OneU8(operand) => self.increment_8_bit(operand),
            CalcedOperands::OneU16(operand) => self.increment_16_bit(operand),
            _ => unreachable!("No other case will get here"),
        }
    }
    fn increment_16_bit(&mut self, operand: u16) {
        let result = operand.wrapping_add(1);
        self.set_instruction_result(result, 0);
    }
    fn increment_8_bit(&mut self, operand: u8) {
        let result = operand.wrapping_add(1);
        self.set_instruction_result(result as u16, 0);
        self.set_flags(
            result == 0,
            false,
            bit_3_overflow(operand, 1),
            self.get_flag(Flag::Carry),
        );
    }

    fn jump_relative(&mut self, operands: CalcedOperands) {
        let signed_offset = match operands {
            CalcedOperands::OneI8(offset) => offset,
            CalcedOperands::CondI8(condition_met, offset) => match condition_met {
                true => offset,
                false => return self.instruction_state_machine.end_instruction_early(),
            },
            _ => unreachable!("No other case will get here"),
        };

        self.pc = self.pc.wrapping_add_signed(signed_offset as i16);
    }

    fn load(&mut self, operands: CalcedOperands) {
        let load_value = match operands {
            CalcedOperands::TwoU8(_, load_value) => load_value as u16,
            CalcedOperands::TwoU16(_, load_value) => load_value,
            _ => unreachable!("Load requires two operands"),
        };
        self.set_instruction_result(load_value, 0);
    }
    fn load_high(&mut self, operands: (u8, u8)) {
        self.set_instruction_result(operands.1 as u16, 0);
    }

    fn or(&mut self, operands: (u8, u8)) {
        let result = operands.0 | operands.1;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, false);
    }

    fn pop(&mut self, value: u16) {
        self.set_instruction_result(value, 0);
    }

    /// Just checks that the condition has been met (if there is one).
    /// If it has not been, it ends the instruction early.
    /// The actual logic for Ret is handled across other M Cycles.
    /// It DOES NOT happen here.
    fn ret(&mut self, condition_met: Option<bool>) {
        if !condition_met.unwrap_or(true) {
            self.instruction_state_machine.end_instruction_early();
        }
    }

    /// Just enables interrupts.
    /// The rest of the logic for RETI is handled across other M Cycles.
    fn reti(&mut self) {
        self.ime = true;
    }

    fn clear_bit(&mut self, operands: (u8, u8)) {
        let mask = !(1 << operands.0);
        let result = operands.1 & mask;

        self.set_instruction_result(result as u16, 1);
    }

    fn rotate_left_through_carry(&mut self, operand: u8) {
        let carry = self.get_flag(Flag::Carry);

        let new_carry = operand >> 7 == 1;
        let result = (operand << 1) | carry as u8;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry);
    }

    fn rotate_left_through_carry_a(&mut self) {
        let a = self.get_a();
        let carry = self.get_flag(Flag::Carry);

        let new_carry = a >> 7 == 1;
        let result = (a << 1) | carry as u8;

        self.set_a(result);
        self.set_flags(false, false, false, new_carry);
    }

    fn rotate_left_into_carry(&mut self, operand: u8) {
        let new_carry = operand >> 7;
        let result = (operand << 1) | new_carry;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry == 1);
    }

    fn rotate_left_into_carry_a(&mut self) {
        let a = self.get_a();

        let new_carry = a >> 7;
        let result = (a << 1) | new_carry;

        self.set_a(result);
        self.set_flags(false, false, false, new_carry == 1);
    }

    fn rotate_right_through_carry(&mut self, operand: u8) {
        let carry = (self.get_flag(Flag::Carry) as u8) << 7;

        let new_carry = (operand & 1) == 1;
        let result = (operand >> 1) | carry;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry);
    }

    fn rotate_right_through_carry_a(&mut self) {
        let a = self.get_a();
        let carry = (self.get_flag(Flag::Carry) as u8) << 7;

        let new_carry = (a & 1) == 1;
        let result = (a >> 1) | carry;

        self.set_a(result);
        self.set_flags(false, false, false, new_carry);
    }

    fn rotate_right_into_carry(&mut self, operand: u8) {
        let new_carry = operand << 7;
        let result = (operand >> 1) | new_carry;

        self.set_instruction_result(result as u16, 0);

        self.set_flags(result == 0, false, false, new_carry != 0);
    }

    fn rotate_right_into_carry_a(&mut self) {
        let a = self.get_a();
        let new_carry = a << 7;
        let result = (a >> 1) | new_carry;

        self.set_a(result);
        self.set_flags(false, false, false, new_carry != 0);
    }

    fn subtract_with_carry(&mut self, operands: (u8, u8)) {
        let carry = self.get_flag(Flag::Carry);

        let (middle_result, overflowed_middle) = operands.0.overflowing_sub(operands.1);
        let (result, overflowed_end) = middle_result.overflowing_sub(carry as u8);

        self.set_instruction_result(result as u16, 0);
        self.set_flags(
            result == 0,
            true,
            bit_4_borrow(operands.0, operands.1, carry),
            overflowed_middle | overflowed_end,
        );
    }

    fn set_carry_flag(&mut self) {
        self.set_flags(self.get_flag(Flag::Zero), false, false, true);
    }

    fn set_bit(&mut self, operands: (u8, u8)) {
        let mask = 1 << operands.0;
        let result = operands.1 | mask;

        self.set_instruction_result(result as u16, 1);
    }

    fn shift_left_arithmetically(&mut self, operand: u8) {
        let new_carry = operand >> 7 == 1;
        let result = operand << 1;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry);
    }

    fn shift_right_arithmetically(&mut self, operand: u8) {
        let sign = operand & 0x80;

        let new_carry = operand & 1 == 1;
        let result = (operand >> 1) | sign;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry);
    }

    fn shift_right_logically(&mut self, operand: u8) {
        let new_carry = operand & 1 == 1;
        let result = operand >> 1;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, new_carry);
    }

    fn stop(&self) {
        notate_event(GameBoyEvent::ChangeGameBoyMode(crate::game_boy::GameBoyMode::Stopped));
    }

    fn subtract(&mut self, operands: (u8, u8)) {
        let (result, borrowed) = operands.0.overflowing_sub(operands.1);

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, true, bit_4_borrow(operands.0, operands.1, false), borrowed);
    }

    fn swap(&mut self, operand: u8) {
        let result = operand.rotate_left(4);

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, false);
    }

    fn xor(&mut self, operands: (u8, u8)) {
        let result = operands.0 ^ operands.1;

        self.set_instruction_result(result as u16, 0);
        self.set_flags(result == 0, false, false, false);
    }
}

enum EightBitRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum SixteenBitRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
}

struct InstructionStateMachine {
    instruction: &'static Instruction,
    operand_0: OperandValue,
    operand_1: OperandValue,
    result: u16,
    step_index: u8,
}

impl InstructionStateMachine {
    fn get_op(&self) -> &MicroOp {
        &self.instruction.steps[self.step_index as usize]
    }

    fn update_to_next_instruction(&mut self, instruction: &'static Instruction) {
        *self = Self { instruction, ..Default::default() };
    }

    fn just_completed_instruction(&self) -> bool {
        self.step_index == self.instruction.steps.len() as u8 - 1
    }

    /// Ends an instruction early. This is just used for false branches on conditional branching.
    /// This is done by setting our step index to the last one in the instruction. A little hacky but it should work.
    fn end_instruction_early(&mut self) {
        self.step_index = self.instruction.steps.len() as u8 - 1;
    }

    fn set_operand(&mut self, value: OperandValue, operand_num: u8) {
        match operand_num {
            0 => self.operand_0 = value,
            1 => self.operand_1 = value,
            _ => unreachable!("We shouldn't have more than two operands"),
        }
    }

    fn get_operand(&self, operand_num: u8) -> OperandValue {
        match operand_num {
            0 => self.operand_0,
            1 => self.operand_1,
            _ => unreachable!("We shouldn't have more than two operands"),
        }
    }

    fn set_operand_msb(&mut self, msb: u8, operand_num: u8) {
        match operand_num {
            0 => self.operand_0.set_msb(msb),
            1 => self.operand_1.set_msb(msb),
            _ => unreachable!("We shouldn't have more than two operands"),
        }
    }
    fn set_operand_lsb(&mut self, lsb: u8, operand_num: u8) {
        match operand_num {
            0 => self.operand_0.set_lsb(lsb),
            1 => self.operand_1.set_lsb(lsb),
            _ => unreachable!("We shouldn't have more than two operands"),
        }
    }

    fn get_calced_operands(&self) -> Option<CalcedOperands> {
        match (self.operand_0, self.operand_1) {
            (OperandValue::U8(U8Operand::Calculated(operand_0)), OperandValue::Unused) => {
                Some(CalcedOperands::OneU8(operand_0))
            },

            (
                OperandValue::U8(U8Operand::Calculated(operand_0)),
                OperandValue::U8(U8Operand::Calculated(operand_1)),
            ) => Some(CalcedOperands::TwoU8(operand_0, operand_1)),

            (OperandValue::U16(U16Operand::Calculated(operand_0)), OperandValue::Unused) => {
                Some(CalcedOperands::OneU16(operand_0))
            },

            (
                OperandValue::U16(U16Operand::Calculated(operand_0)),
                OperandValue::U16(U16Operand::Calculated(operand_1)),
            ) => Some(CalcedOperands::TwoU16(operand_0, operand_1)),

            (OperandValue::Condition(cond), OperandValue::Unused) => Some(CalcedOperands::Cond(cond)),

            (
                OperandValue::U16(U16Operand::Calculated(operand_0)),
                OperandValue::I8(I8Operand::Calculated(operand_1)),
            ) => Some(CalcedOperands::AddSpPlusE8(operand_0, operand_1)),

            (OperandValue::Condition(cond), OperandValue::I8(I8Operand::Calculated(operand_1))) => {
                Some(CalcedOperands::CondI8(cond, operand_1))
            },

            (OperandValue::I8(I8Operand::Calculated(operand_0)), OperandValue::Unused) => {
                Some(CalcedOperands::OneI8(operand_0))
            },

            (OperandValue::Condition(cond), OperandValue::U16(U16Operand::Calculated(address))) => {
                Some(CalcedOperands::CondU16(cond, address))
            },

            (OperandValue::Unused, OperandValue::Unused) => Some(CalcedOperands::NoOperands),
            _ => None,
        }
    }
}

impl Default for InstructionStateMachine {
    fn default() -> Self {
        Self {
            instruction: Instruction::nop(),
            step_index: 0,
            operand_0: Default::default(),
            operand_1: Default::default(),
            result: Default::default(),
        }
    }
}

enum CalcedOperands {
    NoOperands,
    OneU8(u8),
    TwoU8(u8, u8),
    OneU16(u16),
    TwoU16(u16, u16),
    AddSpPlusE8(u16, i8), // Just for Add SP, e8
    Cond(bool),           // Just for conditional returns
    CondI8(bool, i8),     // Just for conditional relative jumps
    OneI8(i8),            // Just for unconditional relative jumps
    CondU16(bool, u16),   // Conditional Calls and Conditional Jumps
}

impl CalcedOperands {
    fn get_u8s(self) -> Option<(u8, u8)> {
        if let CalcedOperands::TwoU8(op0, op1) = self {
            Some((op0, op1))
        } else {
            None
        }
    }
    fn get_u8(self) -> Option<u8> {
        if let CalcedOperands::OneU8(op0) = self {
            Some(op0)
        } else {
            None
        }
    }

    fn get_u16(self) -> Option<u16> {
        if let CalcedOperands::OneU16(op0) = self {
            Some(op0)
        } else {
            None
        }
    }

    /// Gets the condition if there is one.
    /// NOTE: This does NOT get the second operand if there is one. Just the condition
    fn get_cond(self) -> Option<bool> {
        match self {
            CalcedOperands::Cond(cond) | CalcedOperands::CondI8(cond, _) | CalcedOperands::CondU16(cond, _) => {
                Some(cond)
            },
            _ => None,
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum OperandValue {
    #[default]
    NotYetDecoded, // The state before decoding an operand. Effectively a None
    U8(U8Operand),           // An 8-bit value. Either read from memory or an 8-bit register
    Pointer(PointerOperand), // A pointer into memory. Either read from memory or from a 16-bit register
    I8(I8Operand),           // A signed 8-bit value. E.G E8
    Condition(bool), // Condition for Conditional Branches. Conditions can be calculated as soon as they're decoded so we can just store the bool directly
    U16(U16Operand), // 16-bit value. Either read from memory or from a 16-bit register.
    Unused,          // For when there are only 1 or no operands for an instruction
}

impl OperandValue {
    fn try_get_msb(&self) -> Option<u8> {
        if let Self::U16(U16Operand::Calculated(val)) = self {
            Some((*val >> 8) as u8)
        } else {
            None
        }
    }

    fn try_get_lsb(&self) -> Option<u8> {
        if let Self::U16(U16Operand::Calculated(val)) = self {
            Some(*val as u8)
        } else {
            None
        }
    }

    fn set_msb(&mut self, msb: u8) {
        match self {
            OperandValue::Pointer(pointer_operand) => pointer_operand.set_msb(msb),
            OperandValue::U16(u16_operand) => u16_operand.set_msb(msb),
            _ => unreachable!(),
        }
    }
    fn set_lsb(&mut self, lsb: u8) {
        match self {
            OperandValue::Pointer(pointer_operand) => pointer_operand.set_lsb(lsb),
            OperandValue::U16(u16_operand) => u16_operand.set_lsb(lsb),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<OperandValue> for bool {
    type Error = InstructionError;

    fn try_from(value: OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::Condition(cond) => Ok(cond),
            OperandValue::NotYetDecoded => Err(InstructionError::OperandNotYetDecoded),
            _ => Err(InstructionError::WrongOperandType),
        }
    }
}
impl TryFrom<OperandValue> for u16 {
    type Error = InstructionError;

    fn try_from(value: OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::U16(operand) => {
                if let U16Operand::Calculated(val) = operand {
                    Ok(val)
                } else {
                    Err(InstructionError::OperandNotYetCalculated)
                }
            },
            OperandValue::NotYetDecoded => Err(InstructionError::OperandNotYetDecoded),
            _ => Err(InstructionError::WrongOperandType),
        }
    }
}
impl TryFrom<OperandValue> for u8 {
    type Error = InstructionError;

    fn try_from(value: OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::U8(operand) => {
                if let U8Operand::Calculated(val) = operand {
                    Ok(val)
                } else {
                    Err(InstructionError::OperandNotYetCalculated)
                }
            },
            OperandValue::NotYetDecoded => Err(InstructionError::OperandNotYetDecoded),
            _ => Err(InstructionError::WrongOperandType),
        }
    }
}
impl TryFrom<OperandValue> for i8 {
    type Error = InstructionError;

    fn try_from(value: OperandValue) -> Result<Self, Self::Error> {
        match value {
            OperandValue::I8(operand) => {
                if let I8Operand::Calculated(val) = operand {
                    Ok(val)
                } else {
                    Err(InstructionError::OperandNotYetCalculated)
                }
            },
            OperandValue::NotYetDecoded => Err(InstructionError::OperandNotYetDecoded),
            _ => Err(InstructionError::WrongOperandType),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum U8Operand {
    #[default]
    NotCalculated,
    Calculated(u8),
}
#[derive(Default, PartialEq, Clone, Copy, Debug)]
enum U16Operand {
    #[default]
    NotYetCalculated,
    CalculatedLsb(u8),
    CalculatedMsb(u8),
    Calculated(u16),
}

impl U16Operand {
    fn set_lsb(&mut self, lsb: u8) {
        match self {
            U16Operand::Calculated(_) | U16Operand::NotYetCalculated => *self = Self::CalculatedLsb(lsb),
            U16Operand::CalculatedMsb(msb) => *self = Self::Calculated(((*msb as u16) << 8) | lsb as u16),
            U16Operand::CalculatedLsb(_) => {
                unreachable!("")
            },
        }
    }

    fn set_msb(&mut self, msb: u8) {
        match self {
            U16Operand::Calculated(_) | U16Operand::NotYetCalculated => *self = Self::CalculatedMsb(msb),
            U16Operand::CalculatedLsb(lsb) => *self = Self::Calculated(((msb as u16) << 8) | *lsb as u16),
            U16Operand::CalculatedMsb(_) => {
                unreachable!()
            },
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum PointerOperand {
    #[default]
    NotYetCalculated,
    CalculatedLsb(u8),
    CalculatedMsb(u8),
    Calculated(u16),
    HLI(u16), // Special case so that we increment HL after using it
    HLD(u16), // Special case so that we decrement HL after using it
}

impl PointerOperand {
    fn set_lsb(&mut self, lsb: u8) {
        match self {
            PointerOperand::NotYetCalculated => *self = Self::CalculatedLsb(lsb),
            PointerOperand::CalculatedMsb(msb) => *self = Self::Calculated(((*msb as u16) << 8) | lsb as u16),
            _ => {
                unreachable!("This shouldn't be able to be called from these states")
            },
        }
    }

    fn set_msb(&mut self, msb: u8) {
        match self {
            PointerOperand::NotYetCalculated => *self = Self::CalculatedMsb(msb),
            PointerOperand::CalculatedLsb(lsb) => *self = Self::Calculated(((msb as u16) << 8) | *lsb as u16),
            _ => {
                unreachable!("This shouldn't be able to be called from these states")
            },
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum I8Operand {
    #[default]
    NotCalculated,
    Calculated(i8),
}

#[derive(Default, PartialEq)]
enum CpuState {
    PerformingInstruction,
    /// Checks for interrupts before performing the next instruction
    StartingNewInstruction,
    /// Since the next instruction is fetched as the last one completes, this state is only for startup or after stops
    #[default]
    FetchingInstruction,
    InterruptHandler(Interrupt, InterruptStep),
    Halted,
    HaltBug,
}

impl CpuState {}

#[derive(Clone, Copy, Debug, PartialEq)]
enum InterruptStep {
    FirstWait,
    SecondWait,
    PushMsbPCToStack,
    PushLsbPCToStack,
    SetPCToInterrupt,
}

pub struct CpuOperationContext<'a, 'b> {
    cpu: &'a mut Cpu,
    bus: &'b mut Bus,
}

impl<'a, 'b> CpuOperationContext<'a, 'b> {
    pub fn new(cpu: &'a mut Cpu, bus: &'b mut Bus) -> Self {
        Self { cpu, bus }
    }

    fn _print_detailed_operation(&mut self, instruction: &'static Instruction) {
        print!("PC: {} => {} ", self.cpu.pc - 1, instruction.op_code);
        for operand in instruction.operands {
            match operand {
                Operand::A => print!("A(0x{:02x}) ", self.cpu.get_a()),
                Operand::A16 => print!("A16(0x{:04x}) ", self.bus.read_u16(self.cpu.pc)),
                Operand::A16Pointer => {
                    let addr = self.bus.read_u16(self.cpu.pc);
                    print!("(A16)@0x{:04x}(0x{:02x}) ", addr, self.bus.read(addr));
                },
                Operand::AF => print!("AF(0x{:04x}) ", self.cpu.af),
                Operand::B => print!("B(0x{:02x}) ", self.cpu.bc >> 8),
                Operand::BC => print!("BC(0x{:04x}) ", self.cpu.bc),
                Operand::BCPointer => print!("(BC)@0x{:04x}(0x{:02x}) ", self.cpu.bc, self.bus.read(self.cpu.bc)),
                Operand::C => print!("C(0x{:02x}) ", self.cpu.get_c()),
                Operand::D => print!("D(0x{:02x}) ", self.cpu.de >> 8),
                Operand::DE => print!("DE(0x{:04x}) ", self.cpu.de),
                Operand::DEPointer => print!("(DE)@0x{:04x}(0x{:02x}) ", self.cpu.de, self.bus.read(self.cpu.de)),
                Operand::E => print!("E(0x{:02x}) ", self.cpu.get_e()),
                Operand::E8 => print!("E8(0x{:02x}) ", self.bus.read(self.cpu.pc) as i8),
                Operand::FF00OffsetByA8 => {
                    let offset = self.bus.read(self.cpu.pc);
                    let addr = 0xFF00 + offset as u16;
                    print!(
                        "($FF00+0x{:02x})@0x{:04x}(0x{:02x}) ",
                        offset,
                        addr,
                        self.bus.read(addr)
                    );
                },
                Operand::FF00OffsetByC => {
                    let addr = 0xFF00 + self.cpu.get_c() as u16;
                    print!("($FF00+C)@0x{:04x}(0x{:02x}) ", addr, self.bus.read(addr));
                },
                Operand::H => print!("H(0x{:02x}) ", self.cpu.hl >> 8),
                Operand::HL => print!("HL(0x{:04x}) ", self.cpu.hl),
                Operand::HLDPointer => print!("[HL-](0x{:04x}) ", self.cpu.hl),
                Operand::HLIPointer => print!("[HL+](0x{:04x}) ", self.cpu.hl),
                Operand::HLPointer => print!("(HL)@0x{:04x}(0x{:02x}) ", self.cpu.hl, self.bus.read(self.cpu.hl)),
                Operand::Immediate(val) => print!("Imm(0x{:02x}) ", val),
                Operand::L => print!("L(0x{:02x}) ", self.cpu.get_l()),
                Operand::N16 => print!("N16(0x{:04x}) ", self.bus.read_u16(self.cpu.pc)),
                Operand::N8 => print!("N8(0x{:02x}) ", self.bus.read(self.cpu.pc)),
                Operand::Carry => print!("Carry({}) ", self.cpu.get_flag(Flag::Carry)),
                Operand::NotCarry => print!("NotCarry({}) ", !self.cpu.get_flag(Flag::Carry)),
                Operand::NotZero => print!("NotZero({}) ", !self.cpu.get_flag(Flag::Zero)),
                Operand::SP => print!("SP(0x{:04x}) ", self.cpu.sp),
                Operand::Zero => print!("Zero({}) ", self.cpu.get_flag(Flag::Zero)),
            }
        }
        println!()
    }

    fn tick(&mut self) {
        match &self.cpu.state {
            CpuState::PerformingInstruction => self.tick_instruction_micro_op(),
            CpuState::StartingNewInstruction => self.tick_starting_new_instruction(),
            CpuState::FetchingInstruction => self.fetch_next_instruction(),
            CpuState::InterruptHandler(interrupt, step) => self.tick_handling_interrupt(*interrupt, *step),
            CpuState::Halted => self.tick_halted(),
            _ => unreachable!("Remaining states are exited at the end of the cycle they occur"),
        }
    }

    fn tick_halted(&mut self) {
        if self.should_exit_halt() {
            self.cpu.state = CpuState::FetchingInstruction;
            self.tick();
        }
    }

    fn should_exit_halt(&self) -> bool {
        self.bus.try_get_interrupt().is_some()
    }

    fn tick_starting_new_instruction(&mut self) {
        if let Some(interrupt) = self.try_get_interrupt() {
            self.cpu.state = CpuState::InterruptHandler(interrupt, InterruptStep::FirstWait);
            self.tick_handling_interrupt(interrupt, InterruptStep::FirstWait);
        } else {
            self.cpu.state = CpuState::PerformingInstruction;
            self.tick_instruction_micro_op();
        }
    }

    fn try_get_interrupt(&self) -> Option<Interrupt> {
        if self.cpu.interrupts_are_enabled() {
            self.bus.try_get_interrupt()
        } else {
            None
        }
    }

    fn tick_handling_interrupt(&mut self, interrupt: Interrupt, step: InterruptStep) {
        match step {
            InterruptStep::FirstWait => {
                self.cpu.pc -= 1; // to account for the instruction we've already fetched
                self.bus.lower_interrupt_flag(&interrupt);
                self.cpu.disable_interrupts();
                self.cpu.state = CpuState::InterruptHandler(interrupt, InterruptStep::SecondWait);
            },
            InterruptStep::SecondWait => {
                self.cpu.state = CpuState::InterruptHandler(interrupt, InterruptStep::PushMsbPCToStack);
            },
            InterruptStep::PushMsbPCToStack => {
                let msb = (self.cpu.pc >> 8) as u8;
                self.push_to_stack(msb);
                self.cpu.state = CpuState::InterruptHandler(interrupt, InterruptStep::PushLsbPCToStack);
            },
            InterruptStep::PushLsbPCToStack => {
                let lsb = self.cpu.pc as u8;
                self.push_to_stack(lsb);
                self.cpu.state = CpuState::InterruptHandler(interrupt, InterruptStep::SetPCToInterrupt);
            },
            InterruptStep::SetPCToInterrupt => {
                self.cpu.pc = interrupt.get_isr_address();
                self.cpu.state = CpuState::PerformingInstruction;
                self.fetch_next_instruction();
            },
        }
    }

    fn fetch_next_instruction(&mut self) {
        let fetched_byte = self.read_at_pc_and_incr();
        let instruction = &UNPREFIXED[fetched_byte as usize];
        self.cpu.update_to_next_instruction(instruction);
    }
    fn fetch_next_instruction_halt_bug(&mut self) {
        self.fetch_next_instruction();
        self.cpu.pc = self.cpu.pc.wrapping_sub(1);
    }

    fn handle_ei_delay(&mut self) {
        match self.cpu.ei_delay {
            0 => (),
            1 => {
                self.cpu.ei_delay = 0;
                self.cpu.ime = true;
            },
            2 => self.cpu.ei_delay = 1,
            _ => unreachable!("Is never more than 2"),
        }
    }
}

// Methods for Instruction MicroOps
impl<'a, 'b> CpuOperationContext<'a, 'b> {
    fn tick_instruction_micro_op(&mut self) {
        match self.cpu.instruction_state_machine.get_op() {
            MicroOp::Decode => self.cpu.decode_instruction(),
            MicroOp::PopLsb => self.pop_lsb(),
            MicroOp::PopMsb => self.pop_msb(),
            MicroOp::WriteSPLow => self.write_sp_low(),
            MicroOp::WriteSPHigh => self.write_sp_high(),
            MicroOp::PushMsb => self.push_msb(),
            MicroOp::PushLsb => self.push_lsb(),
            MicroOp::ReadIntoOperand0 => self.read_into_operand(0),
            MicroOp::ReadIntoOperand1 => self.read_into_operand(1),
            MicroOp::ReadIntoOperand1Msb => self.read_into_operand_msb(1),
            MicroOp::ReadIntoOperand0Msb => self.read_into_operand_msb(0),
            MicroOp::Write => self.write(),
            MicroOp::Wait => (),
            MicroOp::CbPrefix => return self.cb_prefix(),
            MicroOp::PopStackIntoLsbPc => self.pop_stack_into_lsb_pc(),
            MicroOp::PopStackIntoMsbPc => self.pop_stack_into_msb_pc(),
            MicroOp::PushMsbPCToStackOperand0 => self.push_msb_pc_to_stack(),
            MicroOp::PushLsbPcToStackOperand0 => self.push_lsb_pc_to_stack(0),
            MicroOp::PushMsbPCToStackOperand1 => self.push_msb_pc_to_stack(),
            MicroOp::PushLsbPcToStackOperand1 => self.push_lsb_pc_to_stack(1),
            MicroOp::Illegal => notate_event(GameBoyEvent::TriedRunningIllegalInstruction),
            MicroOp::ReadSPPlusE8 => self.read_sp_plus_e8(),
            MicroOp::ReadIntoOperand0Lsb => self.read_into_operand_lsb(0),
            MicroOp::ReadIntoOperand1Lsb => self.read_into_operand_lsb(1),
            MicroOp::ReadE8Operand0 => self.read_e8(0),
            MicroOp::ReadE8Operand1 => self.read_e8(1),
            MicroOp::WriteBack => self.write_back(),
            MicroOp::CheckForInterrupts => self.check_for_interrupt(),
        }

        if self.cpu.instruction_state_machine.just_completed_instruction() {
            self.handle_ei_delay();

            match self.cpu.state {
                CpuState::Halted => (),
                CpuState::HaltBug => self.fetch_next_instruction_halt_bug(),
                _ => self.fetch_next_instruction(),
            }
        } else {
            self.cpu.instruction_state_machine.step_index += 1;
        }
    }

    fn write_back(&mut self) {
        // writebacks are only done in operations using HL as a pointer
        let address = self.cpu.hl;

        let value = self.cpu.instruction_state_machine.result as u8;

        self.bus.write(address, value);
    }

    fn write(&mut self) {
        let OperandValue::U8(U8Operand::Calculated(value)) = self.cpu.instruction_state_machine.operand_1 else {
            unreachable!("Write is only called in instructions where Op1 is a u8")
        };

        self.write_memory_operand(0, value);
    }

    fn write_sp_low(&mut self) {
        let sp_low = self.cpu.instruction_state_machine.get_operand(1).try_get_lsb().unwrap();
        self.write_memory_operand(0, sp_low);
    }

    fn write_sp_high(&mut self) {
        let sp_high = self.cpu.instruction_state_machine.get_operand(1).try_get_msb().unwrap();

        // LD [n16] SP is a special case so we have to go outside the regular hierarchy and do things by hand
        let OperandValue::Pointer(PointerOperand::Calculated(address)) =
            self.cpu.instruction_state_machine.get_operand(0)
        else {
            unreachable!("The only place this is called is when operand 0 matches the above structure")
        };

        self.bus.write(address + 1, sp_high);
    }

    fn read_into_operand_lsb(&mut self, operand_num: u8) {
        let lsb = self.read_memory_operand(operand_num);
        self.cpu.set_operand_lsb(lsb, operand_num);
    }
    fn read_into_operand_msb(&mut self, operand_num: u8) {
        let msb = self.read_memory_operand(operand_num);
        self.cpu.set_operand_msb(msb, operand_num);
    }

    fn read_into_operand(&mut self, operand_num: u8) {
        let value = self.read_memory_operand(operand_num);
        self.cpu
            .set_instruction_operand(OperandValue::U8(U8Operand::Calculated(value)), operand_num);
    }

    fn push_msb(&mut self) {
        let msb = self.cpu.instruction_state_machine.operand_0.try_get_msb().unwrap();
        self.push_to_stack(msb);
    }
    fn push_lsb(&mut self) {
        let lsb = self.cpu.instruction_state_machine.operand_0.try_get_lsb().unwrap();
        self.push_to_stack(lsb);
    }

    fn read_e8(&mut self, operand_num: u8) {
        let e8 = self.read_memory_operand(operand_num) as i8;
        self.cpu
            .set_instruction_operand(OperandValue::I8(I8Operand::Calculated(e8)), operand_num);
    }

    fn read_sp_plus_e8(&mut self) {
        let e8 = self.read_memory_operand(1) as i8;
        let sp = self.cpu.sp;
        let result = sp.wrapping_add_signed(e8 as i16);
        self.cpu
            .set_instruction_operand(OperandValue::U16(U16Operand::Calculated(result)), 1);
        self.cpu.set_flags(
            false,
            false,
            bit_3_overflow(sp as u8, e8 as u8),
            bit_7_overflow(sp as u8, e8 as u8),
        );
    }

    fn check_for_interrupt(&mut self) {
        let pending_interrupt = self.bus.try_get_interrupt().is_some();
        self.cpu
            .set_instruction_operand(OperandValue::Condition(pending_interrupt), 0);
    }

    fn pop_lsb(&mut self) {
        let popped_value = self.pop_from_stack();
        self.cpu.set_operand_lsb(popped_value, 0);
    }

    fn pop_msb(&mut self) {
        let popped_value = self.pop_from_stack();
        self.cpu.set_operand_msb(popped_value, 0);
    }

    /// Pushes the PC's upper byte to the stack
    fn push_msb_pc_to_stack(&mut self) {
        self.push_to_stack((self.cpu.pc >> 8) as u8);
    }

    fn push_lsb_pc_to_stack(&mut self, operand_num: u8) {
        let new_pc = match self.cpu.instruction_state_machine.get_operand(operand_num) {
            OperandValue::U8(U8Operand::Calculated(value)) => value as u16,
            OperandValue::U16(U16Operand::Calculated(value)) => value,
            _ => unreachable!("There are only three places this is called and it can only be the above values"),
        };

        self.push_to_stack(self.cpu.pc as u8);

        self.cpu.pc = new_pc;
    }

    fn cb_prefix(&mut self) {
        let fetched_byte = self.read_at_pc_and_incr() as usize;
        let instruction = &CBPREFIXED[fetched_byte];
        self.cpu
            .instruction_state_machine
            .update_to_next_instruction(instruction);
        self.cpu.state = CpuState::PerformingInstruction;
    }

    fn pop_stack_into_msb_pc(&mut self) {
        let new_msb_pc = self.pop_from_stack() as u16;

        let mut pc = self.cpu.pc;

        pc &= 0x00FF;
        pc |= new_msb_pc << 8;

        self.cpu.pc = pc;
    }

    fn pop_stack_into_lsb_pc(&mut self) {
        let new_lsb_pc = self.pop_from_stack() as u16;

        let mut pc = self.cpu.pc;
        pc &= 0xFF00;
        pc |= new_lsb_pc;

        self.cpu.pc = pc;
    }

    fn read_at_pc_and_incr(&mut self) -> u8 {
        let pc = self.cpu.pc;
        self.cpu.pc = pc.wrapping_add(1);
        self.bus.read(pc)
    }

    fn pop_from_stack(&mut self) -> u8 {
        let sp = self.cpu.sp;
        let result = self.bus.read(sp);
        self.cpu.sp = sp.wrapping_add(1);
        result
    }

    fn push_to_stack(&mut self, value: u8) {
        let sp = self.cpu.sp.wrapping_sub(1);
        self.cpu.sp = sp;

        self.bus.write(sp, value)
    }

    fn read_memory_operand(&mut self, operand_num: u8) -> u8 {
        // if the operand is a fully formed pointer, return the memory its pointing to.
        // if its HLD or HLI, read from that address and update HL
        // Otherwise, return from PC and increment PC

        if let OperandValue::Pointer(pointer) = self.cpu.instruction_state_machine.get_operand(operand_num) {
            match pointer {
                PointerOperand::Calculated(address) => self.bus.read(address),
                PointerOperand::HLI(address) => {
                    self.cpu.hl = self.cpu.hl.wrapping_add(1);
                    self.bus.read(address)
                },
                PointerOperand::HLD(address) => {
                    self.cpu.hl = self.cpu.hl.wrapping_sub(1);
                    self.bus.read(address)
                },
                _ => self.read_at_pc_and_incr(),
            }
        } else {
            self.read_at_pc_and_incr()
        }
    }

    fn write_memory_operand(&mut self, operand_num: u8, value: u8) {
        if let OperandValue::Pointer(pointer) = self.cpu.instruction_state_machine.get_operand(operand_num) {
            match pointer {
                PointerOperand::Calculated(address) => self.bus.write(address, value),
                PointerOperand::HLI(address) => {
                    self.bus.write(address, value);
                    self.cpu.hl = self.cpu.hl.wrapping_add(1);
                },
                PointerOperand::HLD(address) => {
                    self.bus.write(address, value);
                    self.cpu.hl = self.cpu.hl.wrapping_sub(1);
                },
                _ => unreachable!("There is no operation that will have the state machine call this and fail"),
            }
        } else {
            unreachable!("There is no operation that will have the state machine call this and fail")
        }
    }
}

pub enum Flag {
    Zero,
    Subtraction,
    HalfCarry,
    Carry,
}

impl Flag {
    fn af_index(&self) -> usize {
        match self {
            Flag::Zero => 7,
            Flag::Subtraction => 6,
            Flag::HalfCarry => 5,
            Flag::Carry => 4,
        }
    }
}

#[derive(Debug)]
enum InstructionError {
    OperandNotYetDecoded,
    OperandNotYetCalculated,
    WrongOperandType,
}

fn bit_3_overflow(operand_0: u8, operand_1: u8) -> bool {
    (operand_0 & 0x0F) + (operand_1 & 0x0F) > 0x0F
}
fn bit_7_overflow(operand_0: u8, operand_1: u8) -> bool {
    operand_0 as u16 + operand_1 as u16 > 0xFF
}
fn bit_4_borrow(operand_0: u8, operand_1: u8, carry: bool) -> bool {
    (operand_0 & 0x0F) < ((operand_1 & 0x0F) + carry as u8)
}
