use crate::bus::{Bus, MemoryAccessError};

#[derive(Default)]
pub struct Cpu {
    registers: [u16;6],
}

impl Cpu {
    pub fn new() -> Self {
        Default::default()
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.get_af() >> flag.get_af_index() == 1
    }

    fn get_a(&self) -> u8 {
        (self.get_af() >> 4) as u8
    }
    fn get_b(&self) -> u8 {
        (self.get_bc() >> 4) as u8
    }
    fn get_d(&self) -> u8 {
        (self.get_de() >> 4) as u8
    }
    fn get_h(&self) -> u8 {
        (self.get_hl() >> 4) as u8
    }

    fn get_f(&self) -> u8 {
        (self.get_af() & 0x0F) as u8
    }
    fn get_c(&self) -> u8 {
        (self.get_bc() & 0x0F) as u8
    }
    fn get_e(&self) -> u8 {
        (self.get_de() & 0x0F) as u8
    }
    fn get_l(&self) -> u8 {
        (self.get_hl() & 0x0F) as u8
    }

    fn get_af(&self) -> u16 {
        self.registers[0]
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
    fn get_pc(&self) -> u16 {
        self.registers[5]
    }

    fn get_r8(&self, r8: u8, bus: &mut Bus) -> CpuResult<u8> {
        match r8 {
            0 => Ok(self.get_b()),
            1 => Ok(self.get_c()), 
            2 => Ok(self.get_d()), 
            3 => Ok(self.get_e()), 
            4 => Ok(self.get_h()), 
            5 => Ok(self.get_l()), 
            6 => Ok(bus.read(self.get_hl())?),
            7 => Ok(self.get_a()),
            _ => unreachable!("r8 is represented as a 3-bit bitfield. It cannot be more than 7")
        }
    }
    
    fn get_r16(&self, r16: u8) -> u16 {
        match r16 {
            0 => self.get_bc(),
            1 => self.get_de(), 
            2 => self.get_hl(), 
            3 => self.get_sp(),
            _ => unreachable!("r16 is represented as a 2-bit bitfield. It cannot be more than 3")
        }
    }

    fn get_r16_stk(&self, r16_stk: u8) -> u16 {
        match r16_stk {
            0 => self.get_bc(),
            1 => self.get_de(), 
            2 => self.get_hl(), 
            3 => self.get_af(),
            _ => unreachable!("r16_stk is represented as a 2-bit bitfield. It cannot be more than 3")
        } 
    }

    fn get_r16_mem(&mut self, r16_mem: u8, bus: &mut Bus) -> CpuResult<u16> {
        match r16_mem {
            0 => Ok(self.get_bc()),
            1 => Ok(self.get_de()), 
            2 => {
                let hl = self.get_hl();
                self.set_hl(hl + 1);
                Ok(hl)

            },
            3 => {
                let hl = self.get_hl();
                self.set_hl(hl - 1);
                Ok(hl)

            },
            _ => unreachable!("r16_mem is represented as a 2-bit bitfield. It cannot be more than 3")

        }
    }

}

// Instructions
impl Cpu {
    
}


enum Flag {
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


type CpuResult<T> = Result<T, CpuError>;

enum CpuError {
    MemoryAccessError(MemoryAccessError)
}

impl From<MemoryAccessError> for CpuError {
    fn from(value: MemoryAccessError) -> Self {
        Self::MemoryAccessError(value)
    }
}
