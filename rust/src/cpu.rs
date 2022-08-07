// TODO: Remove this lint rules
#![allow(unused)]

use strum_macros::EnumIter;

#[derive(Default)]
pub struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    program_counter: u16,
    status: u8,
    stack_pointer: u8,
}

impl Cpu {
    pub fn new(
        register_a: u8,
        register_x: u8,
        register_y: u8,
        program_counter: u16,
        status: u8,
        stack_pointer: u8,
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

    pub fn has_flag(&self, flag: &StatusFlag) -> bool {
        (self.status & flag.bit_shift()) != 0
    }

    pub fn enable_flag(&mut self, flag: &StatusFlag) {
        self.status |= flag.bit_shift();
    }

    pub fn disable_flag(&mut self, flag: &StatusFlag) {
        self.status ^= flag.bit_shift();
    }
}



#[derive(EnumIter, Debug)]
pub enum StatusFlag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Constant,
    Overflow,
    Negative,
}

impl StatusFlag {
    pub fn bit_shift(&self) -> u8 {
        match self {
            StatusFlag::Carry => 0x01,
            StatusFlag::Zero => 0x02,
            StatusFlag::Interrupt => 0x04,
            StatusFlag::Decimal => 0x08,
            StatusFlag::Break => 0x10,
            StatusFlag::Constant => 0x20,
            StatusFlag::Overflow => 0x40,
            StatusFlag::Negative => 0x80,
        }
    }
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

#[cfg(test)]
mod test {
    use super::{Cpu, StatusFlag};
    use strum::IntoEnumIterator;

    #[test]
    fn cpu_status_test() {
        for case in StatusFlag::iter() {
            let mut cpu = Cpu::default();
            // All flags are off by default
            assert!(!cpu.has_flag(&case));

            cpu.enable_flag(&case);
            assert!(cpu.has_flag(&case));

            cpu.disable_flag(&case);
            assert!(!cpu.has_flag(&case));
        }
    }
}
