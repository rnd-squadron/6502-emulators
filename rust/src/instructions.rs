#![allow(unused)]

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

    // Common Math opcodes
    // A/S - add/substract, I/D - inc/decrement, A/L - arithmetic/logical shift right
    Adc,
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

    // Common Control Flow opcodes
    Jmp,
    Beq,
    Bne,
}

impl Instruction {
    fn from_byte(byte: u8) -> Instruction { 
        match(byte) { 
            0x69 => Instruction::Adc,
            _ => panic!("Opcode not found! Opcode: {:x}", byte)
        }
    }
} 

#[cfg(test)]
mod opcode_test { 
    use super::Instruction;

    #[test]
    #[should_panic]
    fn instruction_from_byte_test() { 
        Instruction::from_byte(0xff);
    }
}