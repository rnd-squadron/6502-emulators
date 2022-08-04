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

