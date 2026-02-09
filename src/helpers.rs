use core::fmt;
use std::{fs, path::PathBuf};

use crate::processor::{
    instruction_tables::{CBPREFIXED, UNPREFIXED},
    instructions::{Instruction, OpCode},
};

pub fn concat_2_bytes(left: u8, right: u8) -> u16 {
    ((left as u16) << 8) | right as u16
}

/// gets a chunk of bits in a byte specified by the leftmost and rightmost (inclusive) bits desired.
pub fn get_bitfield(mut byte: u8, left: u8, right: u8) -> u8 {
    byte = (byte << (7 - left)) >> (7 - left); // zero out left bits
    byte >>= right; // shift remaining bits to the far right of the return val
    byte
}

/// Maybe we'll make this into an actual log some day but for now, we'll just print it out.
pub fn log(_args: fmt::Arguments) {
    // println!("Log Message: {_args}",)
}

pub fn disassemble_rom(rom_path: &PathBuf, output_path: &PathBuf) {
    let assembly = get_disassembly(&fs::read(rom_path).unwrap());

    fs::write(output_path, assembly).unwrap();
}

fn get_disassembly(bytes: &[u8]) -> String {
    let mut psuedo_pc = 0;
    let mut assembly = Vec::<String>::new();

    while psuedo_pc < bytes.len() {
        let instruction = read_instruction(bytes, &mut psuedo_pc);
        assembly.push(instruction.to_string());
    }

    assembly.join("\n")
}

fn read_instruction(bytes: &[u8], pc: &mut usize) -> &'static Instruction {
    let first_byte = bytes[*pc];

    let unprefixed_instruction = &UNPREFIXED[first_byte as usize];
    let instruction = match unprefixed_instruction.op_code {
        OpCode::Prefix => &CBPREFIXED[bytes[*pc + 1] as usize],
        _ => unprefixed_instruction,
    };

    *pc += instruction.bytes as usize;

    instruction
}

pub struct StackAllocQueue<T: Default + Copy, const MAX_SIZE: usize> {
    queue: [T; MAX_SIZE],
    front_index: u8,
    length: u8,
}

/// Makes sure the provided MAX_SIZE for a StackAllocQueue isn't unreasonably large.
/// This chosen limit more or less arbitrary and based on vibes, as all performant code should be.
/// This can be further refined if there's a reason to.  
const fn _assert_max_capacity(provided_size: usize) {
    assert!(provided_size <= u8::MAX as usize);
}

impl<const SIZE: usize, T: Default + Copy> Default for StackAllocQueue<T, SIZE> {
    fn default() -> Self {
        Self {
            queue: [T::default(); SIZE],
            front_index: Default::default(),
            length: 0,
        }
    }
}

impl<const MAX_SIZE: usize, T: Default + Copy> StackAllocQueue<T, MAX_SIZE> {
    const _MAX_CAPACITY_ASSERTION: () = _assert_max_capacity(MAX_SIZE);

    pub fn pop_unchecked(&mut self) -> T {
        let popped_entry = self.queue[self.front_index as usize];

        self.length -= 1;
        self.front_index = (self.front_index + 1) % MAX_SIZE as u8;

        popped_entry
    }

    pub fn try_pop(&mut self) -> Option<T> {
        match self.length == 0 {
            true => None,
            false => Some(self.pop_unchecked()),
        }
    }

    pub fn push_unchecked(&mut self, new_entry: T) {
        let index = (self.front_index + self.length) % MAX_SIZE as u8;
        self.queue[index as usize] = new_entry;
        self.length += 1;
    }

    pub fn push(&mut self, new_entry: T) -> bool {
        match self.length == MAX_SIZE as u8 {
            true => false,
            false => {
                self.push_unchecked(new_entry);
                true
            },
        }
    }

    pub fn try_get_entry_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length as usize {
            return None;
        }

        let ring_index = (self.front_index as usize + index) % MAX_SIZE;

        Some(&mut self.queue[ring_index])
    }

    pub fn clear_queue(&mut self) {
        self.length = 0;
    }
    pub fn length(&self) -> u8 {
        self.length
    }
}

pub struct RingBuffer<T: Default + Copy, const BUFFER_SIZE: usize> {
    buffer: [T; BUFFER_SIZE],
    write_pos: usize,
    count: usize,
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> Self {
        Self { buffer: [T::default(); BUFFER_SIZE], write_pos: 0, count: 0 }
    }

    pub fn push(&mut self, item: T) {
        self.buffer[self.write_pos] = item;

        self.write_pos = (self.write_pos + 1) % BUFFER_SIZE;

        if self.count < BUFFER_SIZE {
            self.count += 1;
        }
    }
    pub fn as_vec(&self) -> Vec<&T> {
        let (first, second) = if self.count < BUFFER_SIZE {
            (&self.buffer[0..self.write_pos], &[][..])
        } else {
            (&self.buffer[self.write_pos..], &self.buffer[..self.write_pos])
        };

        first.iter().chain(second.iter()).collect()
    }
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> Default for RingBuffer<T, BUFFER_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

/// A representation of an individual bit.
/// It's just a bit more meaningful than a bool.
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum Bit {
    #[default]
    Low,
    High,
}

impl Bit {
    pub fn from_byte(byte: u8) -> [Self; 8] {
        std::array::from_fn(|i| Self::from_lowest_bit_in_byte(byte >> i))
    }

    pub fn to_byte(bits: [&Self; 8]) -> u8 {
        let mut byte = 0;
        for (i, bit) in bits.iter().enumerate() {
            byte |= bit.as_u8() << i
        }
        byte
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Bit::High => 1,
            Bit::Low => 0,
        }
    }

    pub fn from_lowest_bit_in_byte(byte: u8) -> Self {
        match byte & 0b1 {
            0 => Self::Low,
            1 => Self::High,
            _ => unreachable!(),
        }
    }

    pub fn to_char(bits: [&Self; 8]) -> char {
        Bit::to_byte(bits) as char
    }
}

impl From<bool> for Bit {
    fn from(value: bool) -> Self {
        match value {
            true => Bit::High,
            false => Bit::Low,
        }
    }
}
