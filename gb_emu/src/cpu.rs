use crate::{
    bus::{Bus, MemoryAccessError},
    instructions::Operand,
};

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
    fn get_d(&self) -> u8 {
        (self.get_de() >> 8) as u8
    }
    fn get_h(&self) -> u8 {
        (self.get_hl() >> 8) as u8
    }

    fn get_f(&self) -> u8 {
        (self.get_af() & 0xFF) as u8
    }
    fn get_c(&self) -> u8 {
        (self.get_bc() & 0xFF) as u8
    }
    fn get_e(&self) -> u8 {
        (self.get_de() & 0xFF) as u8
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

    fn get_de(&self) -> u16 {
        self.registers[2]
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
    pub fn get_pc(&self) -> u16 {
        self.registers[5]
    }
    pub fn set_pc(&mut self, value: u16) {
        self.registers[5] = value;
    }

    fn get_r8(&self, r8: R8, bus: &mut Bus) -> u8 {
        match r8.into() {
            0 => self.get_b(),
            1 => self.get_c(),
            2 => self.get_d(),
            3 => self.get_e(),
            4 => self.get_h(),
            5 => self.get_l(),
            6 => unreachable!("This should have been handled elsewhere."),
            7 => self.get_a(),
            _ => unreachable!("r8 is represented as a 3-bit bitfield. It cannot be more than 7"),
        }
    }

    fn get_r16(&self, r16: R16) -> u16 {
        match r16.into() {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.get_sp(),
            _ => unreachable!("r16 is represented as a 2-bit bitfield. It cannot be more than 3"),
        }
    }

    fn get_r16_stk(&self, r16_stk: R16Stk) -> u16 {
        match r16_stk.into() {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.get_af(),
            _ => unreachable!("r16_stk is represented as a 2-bit bitfield. It cannot be more than 3"),
        }
    }

    fn get_r16_mem(&mut self, r16_mem: R16Mem) -> u16 {
        match r16_mem.into() {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => {
                let hl = self.get_hl();
                self.set_hl(hl + 1);
                hl
            },
            3 => {
                let hl = self.get_hl();
                self.set_hl(hl - 1);
                hl
            },
            _ => unreachable!("r16_mem is represented as a 2-bit bitfield. It cannot be more than 3"),
        }
    }

    pub fn check_condition(&self, cond: &Condition) -> bool {
        self.get_condition(cond) == 1
    }

    fn get_condition(&self, cond: &Condition) -> u8 {
        match (*cond).into() {
            0 => !self.get_flag(Flag::Zero) & 0b1,
            1 => self.get_flag(Flag::Zero),
            2 => !self.get_flag(Flag::Carry) & 0b1,
            3 => self.get_flag(Flag::Carry),
            _ => unreachable!("cond is represented as a 2-bit bitfield. It cannot be more than 3"),
        }
    }

    pub fn get_operand(&mut self, operand: &Operand, bus: &mut Bus) -> u16 {
        match *operand {
            Operand::R8(r8) => self.get_r8(r8, bus) as u16,
            Operand::R16(r16) => self.get_r16(r16),
            Operand::R16Stk(r16_stk) => self.get_r16_stk(r16_stk),
            Operand::R16Mem(r16_mem) => self.get_r16_mem(r16_mem),
            Operand::N16(val) => val,
            Operand::N8(val) | Operand::U3(val) => val as u16,
            Operand::CC(cond) => self.get_condition(&cond) as u16,
            Operand::SpE8(imm) => self.get_sp() + imm as u16,
            Operand::HlPointer => bus.read(self.get_hl()).unwrap() as u16,
            Operand::MemPointer(address) => bus.read(address).unwrap() as u16,
            Operand::E8(imm) => imm as u16,
        }
    }

    pub fn set_operand(&mut self, operand: &Operand, value: u16, bus: &mut Bus) {
        todo!()
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
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
}

impl TryFrom<u8> for R8 {
    type Error = CpuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::B),
            1 => Ok(Self::C),
            2 => Ok(Self::D),
            3 => Ok(Self::E),
            4 => Ok(Self::H),
            5 => Ok(Self::L),
            7 => Ok(Self::A),
            _ => Err(CpuError::OperandError),
        }
    }
}

impl From<R8> for u8 {
    fn from(value: R8) -> Self {
        match value {
            R8::B => 0,
            R8::C => 1,
            R8::D => 2,
            R8::E => 3,
            R8::H => 4,
            R8::L => 5,
            R8::A => 7,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

impl TryFrom<u8> for R16 {
    type Error = CpuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u16)
    }
}

impl TryFrom<u16> for R16 {
    type Error = CpuError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BC),
            1 => Ok(Self::DE),
            2 => Ok(Self::HL),
            3 => Ok(Self::SP),
            _ => Err(CpuError::OperandError),
        }
    }
}

impl From<R16> for u16 {
    fn from(value: R16) -> Self {
        match value {
            R16::BC => 0,
            R16::DE => 1,
            R16::HL => 2,
            R16::SP => 3,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum R16Stk {
    BC,
    DE,
    HL,
    AF,
}

impl TryFrom<u8> for R16Stk {
    type Error = CpuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u16)
    }
}

impl TryFrom<u16> for R16Stk {
    type Error = CpuError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BC),
            1 => Ok(Self::DE),
            2 => Ok(Self::HL),
            3 => Ok(Self::AF),
            _ => Err(CpuError::OperandError),
        }
    }
}

impl From<R16Stk> for u16 {
    fn from(value: R16Stk) -> Self {
        match value {
            R16Stk::BC => 0,
            R16Stk::DE => 1,
            R16Stk::HL => 2,
            R16Stk::AF => 3,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum R16Mem {
    BC,
    DE,
    HLI,
    HLD,
}

impl TryFrom<u8> for R16Mem {
    type Error = CpuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u16)
    }
}

impl TryFrom<u16> for R16Mem {
    type Error = CpuError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BC),
            1 => Ok(Self::DE),
            2 => Ok(Self::HLI),
            3 => Ok(Self::HLD),
            _ => Err(CpuError::OperandError),
        }
    }
}

impl From<R16Mem> for u16 {
    fn from(value: R16Mem) -> Self {
        match value {
            R16Mem::BC => 0,
            R16Mem::DE => 1,
            R16Mem::HLI => 2,
            R16Mem::HLD => 3,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Condition {}

impl From<Condition> for u8 {
    fn from(value: Condition) -> Self {
        todo!()
    }
}

impl TryFrom<u8> for Condition {
    type Error = CpuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u16)
    }
}

impl TryFrom<u16> for Condition {
    type Error = CpuError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        todo!()
    }
}

type CpuResult<T> = Result<T, CpuError>;

#[derive(Debug)]
pub enum CpuError {
    MemoryAccessError(MemoryAccessError),
    OperandError,
}

impl From<MemoryAccessError> for CpuError {
    fn from(value: MemoryAccessError) -> Self {
        Self::MemoryAccessError(value)
    }
}
