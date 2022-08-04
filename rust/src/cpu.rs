struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    program_counter: u16,
    status: u8,
    stack_pointer: u16,
}

enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX(u8),
    ZeroPageY(u8),
    Relative,
    Absolute,
    AbsoluteX(u8),
    AbsoluteY(u8),
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl Cpu {
    fn new(
        register_a: u8,
        register_x: u8,
        register_y: u8,
        program_counter: u16,
        status: u8,
        stack_pointer: u16,
    ) -> Self {
        Cpu {
            register_a,
            register_x,
            register_y,
            program_counter,
            status,
            stack_pointer,
        }
    }
}
