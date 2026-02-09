use crate::processor::instructions::{OpCode, Operand};

enum CpuSteps {
    OpCode,
    ReadLsbN16,
    ReadMsbN16,
    ReadLsbR16,
    ReadMsbR16,
    ReadData,
    ReadZ,
    ReadW,
    ReadE,
    WriteSPHigh,
    WriteSPLow,
    WriteMsbR16,
    WriteLsbR16,
    ReadN8,
    WriteData,
    WriteN,
    Wait,
    CbPrefix,
    WriteMsbPCPlus3,
    WRiteLsbPCPlus3,
    ReadLsbPc,
    ReadMsbPc,
    WriteMsbPc,
    WriteLSbPc,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum OperandType {
    A,
    R8,
    R16,
    Sp,
    Hl,
    Imm3Bit,
    Cond,
    N8,
    N16,
    N16Pointer,
    HLPointer,
    BCPointer,
    DEPointer,
    None,
    FF00PlusC,
    FF00PlusA8,
    HLDecrementPointer,
    HLIncrementPointer,
    SpPlusE8,
    E8,
}

impl From<Operand> for OperandType {
    fn from(value: Operand) -> Self {
        match value {
            Operand::A => A,
            Operand::A16 => N16,
            Operand::A16Pointer => N16Pointer,
            Operand::BC | Operand::DE | Operand::AF => R16,
            Operand::C | Operand::D | Operand::B | Operand::E | Operand::L | Operand::H => R8,
            Operand::BCPointer => BCPointer,
            Operand::DEPointer => DEPointer,
            Operand::E8 => E8,
            Operand::FF00OffsetByA8 => FF00PlusA8,
            Operand::FF00OffsetByC => FF00PlusC,
            Operand::HL => Hl,
            Operand::HLD => HLDecrementPointer,
            Operand::HLI => HLIncrementPointer,
            Operand::HLPointer => HLPointer,
            Operand::Immediate(_) => Imm3Bit,
            Operand::N16 => N16,
            Operand::N8 => N8,
            Operand::Carry | Operand::NotCarry | Operand::NotZero | Operand::Zero => Cond,
            Operand::SP => Sp,
        }
    }
}

impl From<Option<&Operand>> for OperandType {
    fn from(value: Option<&Operand>) -> Self {
        if let Some(operand) = value {
            Self::from(*operand)
        } else {
            Self::None
        }
    }
}

use CpuSteps::*;
use OpCode::*;
use OperandType::*;

const STEP_TABLE: &[((OpCode, OperandType, OperandType), &[CpuSteps])] = &[
    ((Ld, R8, R8), &[OpCode]),
    ((Ld, R8, A), &[OpCode]),
    ((Ld, A, R8), &[OpCode]),
    ((Ld, A, A), &[OpCode]),
    ((Ld, R8, N8), &[OpCode, ReadN8]),
    ((Ld, A, N8), &[OpCode, ReadN8]),
    ((Ld, R8, HLPointer), &[OpCode, ReadData]),
    ((Ld, A, HLPointer), &[OpCode, ReadData]),
    ((Ld, HLPointer, R8), &[OpCode, WriteData]),
    ((Ld, HLPointer, A), &[OpCode, WriteData]),
    ((Ld, HLPointer, N8), &[OpCode, ReadN8, WriteN]),
    ((Ld, A, BCPointer), &[OpCode, ReadData]),
    ((Ld, A, DEPointer), &[OpCode, ReadData]),
    ((Ld, BCPointer, A), &[OpCode, WriteData]),
    ((Ld, DEPointer, A), &[OpCode, WriteData]),
    ((Ld, A, N16Pointer), &[OpCode, ReadLsbN16, ReadMsbN16, ReadData]),
    ((Ld, N16Pointer, A), &[OpCode, ReadLsbN16, ReadMsbN16, WriteData]),
    ((Ldh, A, FF00PlusC), &[OpCode, ReadData]),
    ((Ldh, FF00PlusC, A), &[OpCode, WriteData]),
    ((Ldh, A, FF00PlusA8), &[OpCode, ReadN8, ReadData]),
    ((Ldh, FF00PlusA8, A), &[OpCode, ReadN8, WriteData]),
    ((Ld, A, HLDecrementPointer), &[OpCode, ReadData]),
    ((Ld, HLDecrementPointer, A), &[OpCode, WriteData]),
    ((Ld, A, HLIncrementPointer), &[OpCode, ReadData]),
    ((Ld, HLIncrementPointer, A), &[OpCode, WriteData]),
    ((Ld, R16, N16), &[OpCode, ReadLsbN16, ReadMsbN16]),
    ((Ld, Hl, N16), &[OpCode, ReadLsbN16, ReadMsbN16]),
    ((Ld, Sp, N16), &[OpCode, ReadLsbN16, ReadMsbN16]),
    ((Ld, N16Pointer, Sp), &[OpCode, ReadZ, ReadW, WriteSPHigh, WriteSPLow]),
    ((Ld, Sp, Hl), &[OpCode, Wait]),
    ((Push, R16, None), &[OpCode, Wait, WriteMsbR16, WriteLsbR16]),
    ((Push, Hl, None), &[OpCode, Wait, WriteMsbR16, WriteLsbR16]),
    ((Pop, R16, None), &[OpCode, ReadLsbR16, ReadMsbR16]),
    ((Pop, Hl, None), &[OpCode, ReadLsbR16, ReadMsbR16]),
    ((Ld, Hl, SpPlusE8), &[OpCode, ReadE, Wait]),
    ((Add, A, R8), &[OpCode]),
    ((Add, A, A), &[OpCode]),
    ((Add, A, HLPointer), &[OpCode, ReadData]),
    ((Add, A, N8), &[OpCode, ReadN8]),
    ((Adc, A, R8), &[OpCode]),
    ((Adc, A, A), &[OpCode]),
    ((Adc, A, HLPointer), &[OpCode, ReadData]),
    ((Adc, A, N8), &[OpCode, ReadN8]),
    ((Sub, A, R8), &[OpCode]),
    ((Sub, A, A), &[OpCode]),
    ((Sub, A, HLPointer), &[OpCode, ReadData]),
    ((Sub, A, N8), &[OpCode, ReadN8]),
    ((Sbc, A, R8), &[OpCode]),
    ((Sbc, A, A), &[OpCode]),
    ((Sbc, A, HLPointer), &[OpCode, ReadData]),
    ((Sbc, A, N8), &[OpCode, ReadN8]),
    ((Cp, A, R8), &[OpCode]),
    ((Cp, A, A), &[OpCode]),
    ((Cp, A, HLPointer), &[OpCode, ReadData]),
    ((Cp, A, N8), &[OpCode, ReadData]),
    ((Inc, R8, None), &[OpCode]),
    ((Inc, A, None), &[OpCode]),
    ((Inc, HLPointer, None), &[OpCode, ReadData, WriteData]),
    ((Dec, R8, None), &[OpCode]),
    ((Dec, A, None), &[OpCode]),
    ((Dec, HLPointer, None), &[OpCode, ReadData, WriteData]),
    ((And, A, R8), &[OpCode]),
    ((And, A, A), &[OpCode]),
    ((And, A, HLPointer), &[OpCode, ReadData]),
    ((And, A, N8), &[OpCode, ReadN8]),
    ((Or, A, R8), &[OpCode]),
    ((Or, A, A), &[OpCode]),
    ((Or, A, HLPointer), &[OpCode, ReadData]),
    ((Or, A, N8), &[OpCode, ReadN8]),
    ((Xor, A, R8), &[OpCode]),
    ((Xor, A, A), &[OpCode]),
    ((Xor, A, HLPointer), &[OpCode, ReadData]),
    ((Xor, A, N8), &[OpCode, ReadN8]),
    ((Ccf, None, None), &[OpCode]),
    ((Scf, None, None), &[OpCode]),
    ((Daa, None, None), &[OpCode]),
    ((Cpl, None, None), &[OpCode]),
    ((Inc, R16, None), &[OpCode, Wait]),
    ((Inc, Hl, None), &[OpCode, Wait]),
    ((Inc, Sp, None), &[OpCode, Wait]),
    ((Dec, R16, None), &[OpCode, Wait]),
    ((Dec, Sp, None), &[OpCode, Wait]),
    ((Dec, Hl, None), &[OpCode, Wait]),
    ((Add, Hl, R16), &[OpCode, Wait]),
    ((Add, Hl, Hl), &[OpCode, Wait]),
    ((Add, Hl, Sp), &[OpCode, Wait]),
    ((Add, Sp, E8), &[OpCode, ReadE, Wait, Wait]),
    ((Rlca, None, None), &[OpCode]),
    ((Rrca, None, None), &[OpCode]),
    ((Rla, None, None), &[OpCode]),
    ((Rra, None, None), &[OpCode]),
    ((Rlc, R8, None), &[CbPrefix, OpCode]),
    ((Rlc, A, None), &[CbPrefix, OpCode]),
    ((Rlc, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Rrc, R8, None), &[CbPrefix, OpCode]),
    ((Rrc, A, None), &[CbPrefix, OpCode]),
    ((Rrc, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Rl, R8, None), &[CbPrefix, OpCode]),
    ((Rl, A, None), &[CbPrefix, OpCode]),
    ((Rl, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Rr, R8, None), &[CbPrefix, OpCode]),
    ((Rr, A, None), &[CbPrefix, OpCode]),
    ((Rr, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Sla, R8, None), &[CbPrefix, OpCode]),
    ((Sla, A, None), &[CbPrefix, OpCode]),
    ((Sla, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Sra, R8, None), &[CbPrefix, OpCode]),
    ((Sra, A, None), &[CbPrefix, OpCode]),
    ((Sra, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Swap, R8, None), &[CbPrefix, OpCode]),
    ((Swap, A, None), &[CbPrefix, OpCode]),
    ((Swap, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Srl, R8, None), &[CbPrefix, OpCode]),
    ((Srl, A, None), &[CbPrefix, OpCode]),
    ((Srl, HLPointer, None), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Bit, Imm3Bit, R8), &[CbPrefix, OpCode]),
    ((Bit, Imm3Bit, A), &[CbPrefix, OpCode]),
    ((Bit, Imm3Bit, HLPointer), &[CbPrefix, OpCode, ReadData]),
    ((Res, Imm3Bit, R8), &[CbPrefix, OpCode]),
    ((Res, Imm3Bit, A), &[CbPrefix, OpCode]),
    ((Res, Imm3Bit, HLPointer), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Set, Imm3Bit, R8), &[CbPrefix, OpCode]),
    ((Set, Imm3Bit, A), &[CbPrefix, OpCode]),
    ((Set, Imm3Bit, HLPointer), &[CbPrefix, OpCode, ReadData, WriteData]),
    ((Jp, N16, None), &[OpCode, ReadLsbN16, ReadMsbN16, Wait]),
    ((Jp, Hl, None), &[OpCode]),
    ((Jp, Cond, N16), &[OpCode, ReadLsbN16, ReadMsbN16, Wait]), // THIS IS WHEN CONDITION IS TRUE
    ((Jp, Cond, N16), &[OpCode, ReadLsbN16, ReadMsbN16]),       // THIS IS WHEN CONDITION IS FALSE
    ((Jr, E8, None), &[OpCode, ReadE, Wait]),
    ((Jr, Cond, E8), &[OpCode, ReadE, Wait]), // THIS IS WHEN CONDITION IS TRUE
    ((Jr, Cond, E8), &[OpCode, ReadE]),       // THIS IS WHEN CONDITION IS FALSE
    (
        (Call, N16, None),
        &[OpCode, ReadLsbN16, ReadMsbN16, Wait, WriteMsbPCPlus3, WRiteLsbPCPlus3],
    ),
    (
        // THIS IS WHEN CONDITION IS TRUE
        (Call, Cond, N16),
        &[OpCode, ReadLsbN16, ReadMsbN16, Wait, WriteMsbPCPlus3, WRiteLsbPCPlus3],
    ),
    ((Call, Cond, N16), &[OpCode, ReadLsbN16, ReadMsbN16]), // THIS IS WHEN CONDITION IS FALSE
    ((Ret, None, None), &[OpCode, ReadLsbPc, ReadMsbPc, Wait]),
    ((Ret, Cond, None), &[OpCode, Wait, ReadLsbPc, ReadMsbPc, Wait]), // THIS IS WHEN CONDITION IS TRUE
    ((Ret, Cond, None), &[OpCode, Wait]),                             // THIS IS WHEN CONDITION IS FALSE
    ((Reti, None, None), &[OpCode, ReadLsbPc, ReadMsbPc, Wait]),
    ((Rst, Imm3Bit, None), &[OpCode, Wait, WriteMsbPc, WriteLSbPc]),
    ((Halt, None, None), &[OpCode]),
    ((Stop, None, None), &[OpCode]),
    ((Di, None, None), &[OpCode]),
    ((Ei, None, None), &[OpCode]),
    ((Nop, None, None), &[OpCode]),
];

#[cfg(test)]
mod test {
    use crate::processor::{
        instruction_tables::{CBPREFIXED, UNPREFIXED},
        instructions::Instruction,
        new_cpu::{CpuSteps, OpCode, OperandType, STEP_TABLE},
    };

    fn is_a_match(instruction: &Instruction, step_table_entry: &(OpCode, OperandType, OperandType)) -> bool {
        instruction.op_code == step_table_entry.0
            && OperandType::from(instruction.operands.get(0)) == step_table_entry.1
            && OperandType::from(instruction.operands.get(1)) == step_table_entry.2
    }

    fn get_step_table_entry(instruction: &Instruction) -> &'static [CpuSteps] {
        println!("Checking for {instruction}");
        for entry in STEP_TABLE.iter().rev() {
            if is_a_match(instruction, &entry.0) {
                return entry.1;
            }
        }
        panic!()
    }

    fn is_add_sp_e8(instruction: &Instruction) -> bool {
        instruction.operands.len() == 3
            && instruction.operands[1] == crate::processor::instructions::Operand::SP
            && instruction.operands[2] == crate::processor::instructions::Operand::E8
    }

    #[test]
    fn test_step_table_instr_lengths() {
        for i in (0..256).rev() {
            let instruction = &UNPREFIXED[i];
            if instruction.op_code == OpCode::Illegal
                || instruction.op_code == OpCode::Prefix
                || is_add_sp_e8(instruction)
            {
                continue;
            }
            let table_entry = get_step_table_entry(instruction);
            assert_eq!(instruction.cycles, table_entry.len() as u8)
        }
        for i in (0..256).rev() {
            let instruction = &CBPREFIXED[i];
            let table_entry = get_step_table_entry(instruction);
            assert_eq!(instruction.cycles, table_entry.len() as u8)
        }
    }
}
