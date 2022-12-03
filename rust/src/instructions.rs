#![allow(unused)]

use std::{collections::HashMap, iter::Cycle};

use crate::cpu::{AddressingMode, Cpu, Nes, StatusFlag};

enum Instruction {
    // Common Load/Store opcodes
    Lda,
    Ldx,
    Ldy,
    Sta,
    Stx,
    Sty,

    // Common Transfer opcodes
    Tay,
    Tya,
    Tax,
    Txa,
    Txs,
    Tsx,

    // Common Math opcodes
    // A/S - add/substract, I/D - inc/decrement, A/L - arithmetic/logical shift right
    Adc,
    And,
    Sbc,
    Sec,
    Inc,
    Dec,
    Iny,
    Inx,
    Dey,
    Dex,
    Asl,
    Lsr,

    // Common Comparison opcodes
    Cmp,
    Cpx,
    Cpy,
    Eor,
    Ror,
    Ora,
    Rol,

    // Common Control Flow opcodes
    Jmp,
    Jsr,
    Beq,
    Bne,
}

struct OpCode {
    code: u8,
    bytes: u8,
    cycles: u8,
    instruction: Instruction,
    address_mode: AddressingMode,
}

impl OpCode {
    fn new(
        code: u8,
        instruction: Instruction,
        bytes: u8,
        cycles: u8,
        mode: AddressingMode,
    ) -> OpCode {
        OpCode {
            code,
            bytes,
            cycles,
            instruction,
            address_mode: mode,
        }
    }

    #[rustfmt::skip]
    fn from_byte(code: u8) -> OpCode {
        match(code) {
            // ADC - Add Memory to Accumulator with Carry
            0x69 => OpCode::new(code, Instruction::Adc, 2, 2, AddressingMode::Immediate),
            0x65 => OpCode::new(code, Instruction::Adc, 2, 3, AddressingMode::ZeroPage),
            0x75 => OpCode::new(code, Instruction::Adc, 2, 4, AddressingMode::ZeroPageX),
            0x6D => OpCode::new(code, Instruction::Adc, 3, 4, AddressingMode::Absolute),
            0x7D => OpCode::new(code, Instruction::Adc, 3, 4, AddressingMode::AbsoluteX), // *
            0x79 => OpCode::new(code, Instruction::Adc, 3, 4, AddressingMode::AbsoluteY), // *
            0x61 => OpCode::new(code, Instruction::Adc, 2, 6, AddressingMode::IndexedIndirectX),
            0x71 => OpCode::new(code, Instruction::Adc, 2, 5, AddressingMode::IndirectIndexedY), // *
            // AND - AND Memory with Accumulator
            0x29 => OpCode::new(code, Instruction::And, 2, 2, AddressingMode::Immediate),
            0x25 => OpCode::new(code, Instruction::And, 2, 3, AddressingMode::ZeroPage),
            0x35 => OpCode::new(code, Instruction::And, 2, 4, AddressingMode::ZeroPageX),
            0x2D => OpCode::new(code, Instruction::And, 3, 4, AddressingMode::Absolute),
            0x3D => OpCode::new(code, Instruction::And, 3, 4, AddressingMode::AbsoluteX), // *
            0x39 => OpCode::new(code, Instruction::And, 3, 4, AddressingMode::AbsoluteY), // *
            0x21 => OpCode::new(code, Instruction::And, 2, 6, AddressingMode::IndexedIndirectX),
            0x31 => OpCode::new(code, Instruction::And, 2, 5, AddressingMode::IndexedIndirectX), // *
            // ASL - Shift Left One Bit (Memory or Accumulator)
            0x0A => OpCode::new(code, Instruction::Asl, 1, 2, AddressingMode::Accumulator),
            0x06 => OpCode::new(code, Instruction::Asl, 2, 5, AddressingMode::ZeroPage),
            0x16 => OpCode::new(code, Instruction::Asl, 2, 6, AddressingMode::ZeroPageX),
            0x0E => OpCode::new(code, Instruction::Asl, 3, 6, AddressingMode::Absolute),
            0x1E => OpCode::new(code, Instruction::Asl, 3, 7, AddressingMode::AbsoluteX),
            // CMP - Compare Memory with Accumulator
            0xC9 => OpCode::new(code, Instruction::Cmp, 2, 2, AddressingMode::Immediate),
            0xC5 => OpCode::new(code, Instruction::Cmp, 2, 3, AddressingMode::ZeroPage),
            0xD5 => OpCode::new(code, Instruction::Cmp, 2, 4, AddressingMode::ZeroPageX),
            0xCD => OpCode::new(code, Instruction::Cmp, 3, 4, AddressingMode::Absolute),
            0xDD => OpCode::new(code, Instruction::Cmp, 3, 4, AddressingMode::AbsoluteX), // *
            0xD9 => OpCode::new(code, Instruction::Cmp, 3, 4, AddressingMode::AbsoluteY), // *
            0xC1 => OpCode::new(code, Instruction::Cmp, 2, 6, AddressingMode::IndexedIndirectX),
            0xD1 => OpCode::new(code, Instruction::Cmp, 2, 5, AddressingMode::IndirectIndexedY), // *
            // CPX - Compare Memory and Index X
            0xE0 => OpCode::new(code, Instruction::Cpx, 2, 2, AddressingMode::Immediate),
            0xE4 => OpCode::new(code, Instruction::Cpx, 2, 3, AddressingMode::ZeroPage),
            0xEC => OpCode::new(code, Instruction::Cpx, 3, 4, AddressingMode::Absolute),
            // CPY - Compare Memory and Index Y
            0xC0 => OpCode::new(code, Instruction::Cpy, 2, 2, AddressingMode::Immediate),
            0xC4 => OpCode::new(code, Instruction::Cpy, 2, 3, AddressingMode::ZeroPage),
            0xCC => OpCode::new(code, Instruction::Cpy, 3, 4, AddressingMode::Absolute),
            // DEC - Decrement Memory by One
            0xC6 => OpCode::new(code, Instruction::Dec, 2, 5, AddressingMode::ZeroPage),
            0xD6 => OpCode::new(code, Instruction::Dec, 2, 6, AddressingMode::ZeroPageX),
            0xCE => OpCode::new(code, Instruction::Dec, 3, 6, AddressingMode::Absolute),
            0xDE => OpCode::new(code, Instruction::Dec, 3, 7, AddressingMode::AbsoluteX),
            // EOR - Exclusive-OR Memory with Accumulator 
            0x49 => OpCode::new(code, Instruction::Eor, 2, 2, AddressingMode::Immediate),
            0x45 => OpCode::new(code, Instruction::Eor, 2, 3, AddressingMode::ZeroPage),
            0x55 => OpCode::new(code, Instruction::Eor, 2, 4, AddressingMode::ZeroPageX),
            0x4D => OpCode::new(code, Instruction::Eor, 3, 4, AddressingMode::Absolute),
            0x5D => OpCode::new(code, Instruction::Eor, 3, 4, AddressingMode::AbsoluteX), // *
            0x59 => OpCode::new(code, Instruction::Eor, 3, 4, AddressingMode::AbsoluteY), // *
            0x41 => OpCode::new(code, Instruction::Eor, 2, 6, AddressingMode::IndexedIndirectX),
            0x51 => OpCode::new(code, Instruction::Eor, 2, 5, AddressingMode::IndirectIndexedY), // *
            // INC - Increment Index Y by One
            0xE6 => OpCode::new(code, Instruction::Inc, 2, 5, AddressingMode::ZeroPage),
            0xF6 => OpCode::new(code, Instruction::Inc, 2, 6, AddressingMode::ZeroPageX),
            0xEE => OpCode::new(code, Instruction::Inc, 3, 6, AddressingMode::Absolute),
            0xFE => OpCode::new(code, Instruction::Inc, 3, 7, AddressingMode::AbsoluteX),
            // JMP - Jump to New Location
            0x4C => OpCode::new(code, Instruction::Jmp, 3, 3, AddressingMode::Absolute),
            0x6C => OpCode::new(code, Instruction::Jmp, 3, 5, AddressingMode::Indirect),
            // JSR - Jump to New Location Saving Return Address
            0x20 => OpCode::new(code, Instruction::Jsr, 3, 6, AddressingMode::Absolute),
            // LDA - Load Accumulator with Memory
            0xA9 => OpCode::new(code, Instruction::Lda, 2, 2, AddressingMode::Immediate),
            0xA5 => OpCode::new(code, Instruction::Lda, 2, 3, AddressingMode::ZeroPage),
            0xB5 => OpCode::new(code, Instruction::Lda, 2, 4, AddressingMode::ZeroPageX),
            0xAD => OpCode::new(code, Instruction::Lda, 3, 4, AddressingMode::Absolute),
            0xBD => OpCode::new(code, Instruction::Lda, 3, 4, AddressingMode::AbsoluteX), // *
            0xB9 => OpCode::new(code, Instruction::Lda, 3, 4, AddressingMode::AbsoluteY), // *
            0xA1 => OpCode::new(code, Instruction::Lda, 2, 6, AddressingMode::IndexedIndirectX),
            0xB1 => OpCode::new(code, Instruction::Lda, 2, 5, AddressingMode::IndirectIndexedY), // *
            // LDX - Load Index X with Memory
            0xA2 => OpCode::new(code, Instruction::Ldx, 2, 2, AddressingMode::Immediate),
            0xA6 => OpCode::new(code, Instruction::Ldx, 2, 3, AddressingMode::ZeroPage),
            0xB6 => OpCode::new(code, Instruction::Ldx, 2, 4, AddressingMode::ZeroPageY),
            0xAE => OpCode::new(code, Instruction::Ldx, 3, 4, AddressingMode::Absolute),
            0xBE => OpCode::new(code, Instruction::Ldx, 3, 4, AddressingMode::AbsoluteY), // *
            // LDY - Load Index Y with Memory
            0xA0 => OpCode::new(code, Instruction::Ldy, 2, 2, AddressingMode::Immediate),
            0xA4 => OpCode::new(code, Instruction::Ldy, 2, 3, AddressingMode::ZeroPage),
            0xB4 => OpCode::new(code, Instruction::Ldy, 2, 4, AddressingMode::ZeroPageX),
            0xAC => OpCode::new(code, Instruction::Ldy, 3, 4, AddressingMode::Absolute),
            0xBC => OpCode::new(code, Instruction::Ldy, 3, 4, AddressingMode::AbsoluteX), // *
            // LSR - Shift One Bit Right (Memory or Accumulator)
            0x4A => OpCode::new(code, Instruction::Lsr, 1, 2, AddressingMode::Accumulator),
            0x46 => OpCode::new(code, Instruction::Lsr, 2, 5, AddressingMode::ZeroPage),
            0x56 => OpCode::new(code, Instruction::Lsr, 2, 6, AddressingMode::ZeroPageX),
            0x4E => OpCode::new(code, Instruction::Lsr, 3, 6, AddressingMode::Absolute),
            0x5E => OpCode::new(code, Instruction::Lsr, 3, 7, AddressingMode::AbsoluteX),
            // ORA - OR Memory with Accumulator
            0x09 => OpCode::new(code, Instruction::Ora, 2, 2, AddressingMode::Immediate),
            0x05 => OpCode::new(code, Instruction::Ora, 2, 3, AddressingMode::ZeroPage),
            0x15 => OpCode::new(code, Instruction::Ora, 2, 4, AddressingMode::ZeroPageX),
            0x0D => OpCode::new(code, Instruction::Ora, 3, 4, AddressingMode::Absolute),
            0x1D => OpCode::new(code, Instruction::Ora, 3, 4, AddressingMode::AbsoluteX), // *
            0x19 => OpCode::new(code, Instruction::Ora, 3, 4, AddressingMode::AbsoluteY), // *
            0x01 => OpCode::new(code, Instruction::Ora, 2, 6, AddressingMode::IndexedIndirectX),
            0x11 => OpCode::new(code, Instruction::Ora, 2, 5, AddressingMode::IndirectIndexedY), // *
            // ROL - Rotate One Bit Left (Memory or Accumulator)
            0x2A => OpCode::new(code, Instruction::Rol, 1, 1, AddressingMode::Accumulator),
            0x26 => OpCode::new(code, Instruction::Rol, 2, 2, AddressingMode::ZeroPage),
            0x36 => OpCode::new(code, Instruction::Rol, 2, 2, AddressingMode::ZeroPageX),
            0x2E => OpCode::new(code, Instruction::Rol, 3, 3, AddressingMode::Absolute),
            0x3E => OpCode::new(code, Instruction::Rol, 3, 3, AddressingMode::AbsoluteX),
            // ROR - Rotate One Bit Right (Memory or Accumulator)
            0x6A => OpCode::new(code, Instruction::Ror, 1, 2, AddressingMode::Accumulator),
            0x66 => OpCode::new(code, Instruction::Ror, 2, 5, AddressingMode::ZeroPage),
            0x76 => OpCode::new(code, Instruction::Ror, 2, 6, AddressingMode::ZeroPageX),
            0x6E => OpCode::new(code, Instruction::Ror, 3, 6, AddressingMode::Absolute),
            0x7E => OpCode::new(code, Instruction::Ror, 3, 7, AddressingMode::AbsoluteX),
            // SBC - Subtract Memory from Accumulator with Borrow
            0xE9 => OpCode::new(code, Instruction::Sbc, 2, 2, AddressingMode::Immediate),
            0xE5 => OpCode::new(code, Instruction::Sbc, 2, 3, AddressingMode::ZeroPage),
            0xF5 => OpCode::new(code, Instruction::Sbc, 2, 4, AddressingMode::ZeroPageX),
            0xED => OpCode::new(code, Instruction::Sbc, 3, 4, AddressingMode::Absolute),
            0xFD => OpCode::new(code, Instruction::Sbc, 3, 4, AddressingMode::AbsoluteX), // *
            0xF9 => OpCode::new(code, Instruction::Sbc, 3, 4, AddressingMode::AbsoluteY), // *
            0xE1 => OpCode::new(code, Instruction::Sbc, 2, 6, AddressingMode::IndexedIndirectX),
            0xF1 => OpCode::new(code, Instruction::Sbc, 2, 5, AddressingMode::IndirectIndexedY), // *
            // STA - Store Accumulator in Memory
            0x85 => OpCode::new(code, Instruction::Sta, 2, 3, AddressingMode::ZeroPage),
            0x95 => OpCode::new(code, Instruction::Sta, 2, 4, AddressingMode::ZeroPageX),
            0x8D => OpCode::new(code, Instruction::Sta, 3, 4, AddressingMode::Absolute),
            0x9D => OpCode::new(code, Instruction::Sta, 3, 5, AddressingMode::AbsoluteX),
            0x99 => OpCode::new(code, Instruction::Sta, 3, 5, AddressingMode::AbsoluteY),
            0x81 => OpCode::new(code, Instruction::Sta, 2, 6, AddressingMode::IndexedIndirectX),
            0x91 => OpCode::new(code, Instruction::Sta, 2, 6, AddressingMode::IndirectIndexedY),
            // STX - Store Index X in Memory
            0x86 => OpCode::new(code, Instruction::Stx, 2, 3, AddressingMode::ZeroPage),
            0x96 => OpCode::new(code, Instruction::Stx, 2, 4, AddressingMode::ZeroPageY),
            0x8E => OpCode::new(code, Instruction::Stx, 3, 4, AddressingMode::Absolute),
            // STY - Store Index Y in Memory
            0x84 => OpCode::new(code, Instruction::Sty, 2, 3, AddressingMode::ZeroPage),
            0x94 => OpCode::new(code, Instruction::Sty, 2, 4, AddressingMode::ZeroPageX),
            0x8C => OpCode::new(code, Instruction::Sty, 3, 4, AddressingMode::Absolute),
            _ => panic!("Opcode not found! Opcode: {:x}", code)
        }
    }

    fn lda(self, nes: &mut Nes) {
        let address = nes.get_operand_address(self.address_mode);
        let value = nes.mem_read_8(address);

        nes.cpu.accumulator = value;
        nes.cpu.update_flag(&StatusFlag::Zero, value == 0);
        nes.cpu.update_flag(&StatusFlag::Negative, value >> 7 == 1);
    }
}

#[cfg(test)]
mod opcode_test {
    use super::{Instruction, OpCode};

    #[test]
    #[should_panic]
    fn instruction_from_byte_test() {
        OpCode::from_byte(0xff);
    }
}
