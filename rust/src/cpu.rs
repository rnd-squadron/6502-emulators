// TODO: Remove this lint rules
#![allow(unused)]

use std::{cmp, fs, io::Read, slice};
use strum_macros::EnumIter;

use crate::instructions::OpCode;

pub struct Nes {
    pub cpu: Cpu,
    pub memory: [u8; 0xFFFF], // 64 Kib
}

impl Default for Nes {
    fn default() -> Self {
        Nes {
            cpu: Cpu::default(),
            memory: [0; 0xFFFF],
        }
    }
}

impl Nes {
    pub fn new(cpu: Cpu) -> Self {
        Nes {
            cpu,
            memory: [0; 0xFFFF],
        }
    }

    pub fn reset(&mut self) {
        self.cpu.accumulator = 0;
        self.cpu.register_x = 0;
        self.cpu.register_y = 0;
        self.cpu.status = 0;

        // Reset vector: read from $FFFC and $FFFD
        self.cpu.program_counter = self.mem_read_16(0xFFFC);
    }

    pub fn set_program_counter(&mut self, address: u16) {
        self.cpu.program_counter = address;
    }

    pub fn load(&mut self, data: [u8; 0xFFFF]) {
        self.memory = data;
    }

    pub fn load_rom_from_bytes(&mut self, data: &[u8]) {
        // TODO: fix overflow
        self.memory[0x8000..0x8000 + data.len()].copy_from_slice(data);
    }

    pub fn load_rom_from_file(&mut self, filename: String) {
        let file = fs::File::open(&filename).expect("File not found");

        let data: Vec<u8> = file
            .bytes()
            .take(0x8000)
            .collect::<Result<Vec<u8>, _>>()
            .expect("Error processing byte stream for ROM");

        self.load_rom_from_bytes(&data);
    }

    pub fn mem_read_8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn mem_write_8(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    pub fn mem_read_16(&self, address: u16) -> u16 {
        let low = self.mem_read_8(address) as u16;
        let high = self.mem_read_8(address.wrapping_add(1)) as u16;

        (high << 8) | low
    }

    pub fn mem_write_16(&mut self, address: u16, data: u16) {
        let [high, low] = [(data >> 8) as u8, (data & 0xFF) as u8];

        self.mem_write_8(address, low);
        self.mem_write_8(address.wrapping_add(1), high);
    }

    pub fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        let program_counter = self.cpu.program_counter;

        match mode {
            AddressingMode::Accumulator => self.cpu.accumulator as u16,
            AddressingMode::Immediate => program_counter,
            AddressingMode::ZeroPage => self.mem_read_8(program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let position = self.mem_read_8(program_counter);
                position.wrapping_add(self.cpu.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                let position = self.mem_read_8(program_counter);
                position.wrapping_add(self.cpu.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_16(program_counter),
            AddressingMode::AbsoluteX => {
                let position = self.mem_read_16(program_counter);
                position.wrapping_add(self.cpu.register_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let position = self.mem_read_16(program_counter);
                position.wrapping_add(self.cpu.register_y as u16)
            }
            AddressingMode::Indirect => {
                let address = self.mem_read_16(program_counter);

                u16::from_le(address)
            }
            AddressingMode::IndexedIndirectX => {
                let start_address = self.mem_read_8(program_counter);
                let address = start_address.wrapping_add(self.cpu.register_x) as u16;

                let low = self.mem_read_8(address);
                let high = self.mem_read_8(address.wrapping_add(1));

                u16::from_le_bytes([low, high])
            }
            AddressingMode::IndirectIndexedY => {
                let address = self.mem_read_8(program_counter) as u16;

                let low = self.mem_read_8(address);
                let high = self.mem_read_8(address.wrapping_add(1));

                u16::from_le_bytes([low, high]).wrapping_add(self.cpu.register_y as u16)
            }
        }
    }

    //Operations for transferring bytes of data
    fn lda(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn ldx(&mut self, opcode: OpCode) {
        let adderss = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(adderss);

        self.cpu.register_x = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn ldy(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.register_y = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn sta(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);

        self.mem_write_8(address, self.cpu.accumulator);
    }

    fn stx(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);

        self.mem_write_8(address, self.cpu.register_x)
    }

    fn sty(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);

        self.mem_write_8(address, self.cpu.register_y)
    }

    fn tax(&mut self, opcode: OpCode) {
        self.cpu.register_x = self.cpu.accumulator;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_x);
    }

    fn tay(&mut self, opcode: OpCode) {
        self.cpu.register_y = self.cpu.accumulator;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_y);
    }

    fn txa(&mut self, opcode: OpCode) {
        self.cpu.accumulator = self.cpu.register_x;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn tya(&mut self, opcode: OpCode) {
        self.cpu.accumulator = self.cpu.register_y;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn txs(&mut self, opcode: OpCode) {
        self.cpu.stack_pointer = self.cpu.register_x;
    }

    fn tsx(&mut self, opcode: OpCode) {
        self.cpu.register_x = self.cpu.stack_pointer;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_x);
    }

    // Addition
    fn adc(&mut self, opcode: OpCode) {}

    // Subtraction
    fn sub(&mut self, opcode: OpCode) {}

    // Bitwise operations
    fn and(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator &= value;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn ora(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator |= value;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn eor(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator ^= value;
        self.cpu.update_zero_and_negative_flags(self.cpu.accumulator);
    }

    // Operations for incrementing and decrementing the index registers
    fn inx(&mut self, opcode: OpCode) {
        let (result, _) = self.cpu.register_x.overflowing_add(1);

        self.cpu.register_x = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn iny(&mut self, opcode: OpCode) {
        let (result, _) = self.cpu.register_y.overflowing_add(1);

        self.cpu.register_y = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dex(&mut self, opcode: OpCode) {
        let (result, _) = self.cpu.register_x.overflowing_sub(1);

        self.cpu.register_x = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dey(&mut self, opcode: OpCode) {
        let (result, _) = self.cpu.register_y.overflowing_sub(1);

        self.cpu.register_y = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    // Operations for incrementing and decrementing memory
    fn inc(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);
        let (result, _) = value.overflowing_add(1);

        self.mem_write_8(address, result);
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dec(&mut self, opcode: OpCode) {
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);
        let (result, _) = value.overflowing_sub(1);

        self.mem_write_8(address, result);
        self.cpu.update_zero_and_negative_flags(result);
    }

    // Operations for byte comparison
    fn cmp(&mut self, opcode: OpCode) { 
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.accumulator.wrapping_sub(value);

        self.cpu.update_flag(&StatusFlag::Carry, self.cpu.accumulator >= value);
        self.cpu.update_zero_and_negative_flags(result);
    } 

    fn cpx(&mut self, opcode: OpCode) { 
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.register_x.wrapping_sub(value);

        self.cpu.update_flag(&StatusFlag::Carry, self.cpu.register_x >= value);
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn cpy(&mut self, opcode: OpCode) { 
        let address = self.get_operand_address(opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.register_y.wrapping_sub(value);

        self.cpu.update_flag(&StatusFlag::Carry, self.cpu.register_y >= value);
        self.cpu.update_zero_and_negative_flags(result);
    }
}

#[derive(Default)]
pub struct Cpu {
    pub accumulator: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub status: u8,
    pub stack_pointer: u8,
}

impl Cpu {
    pub fn new(
        accumulator: u8,
        register_x: u8,
        register_y: u8,
        program_counter: u16,
        status: u8,
        stack_pointer: u8,
    ) -> Self {
        Cpu {
            accumulator,
            register_x,
            register_y,
            program_counter,
            status,
            stack_pointer,
        }
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
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

    pub fn update_flag(&mut self, flag: &StatusFlag, is_enable: bool) {
        if is_enable {
            self.enable_flag(flag)
        } else {
            self.disable_flag(flag)
        }
    }

    pub fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.update_flag(&StatusFlag::Zero, value == 0);
        self.update_flag(&StatusFlag::Negative, value >> 7 == 1);
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

/// Instructions need operands to work on.
/// Addressing modes are various ways that indicate where
/// the processor should receive these operands.
///
/// The list includes only modes that are common to many instructions.
///
/// Other modes are specific to specific instructions, namely:
/// - Implicit: In this mode the operand's value is given in the instruction itself;
/// - Accumulator: In this mode the instruction operates on data in the
/// accumulator, so no operands are needed;
/// - Relative: This mode is used with Branch-on-Condition instructions.
/// - Indirect: This mode applies only to the JMP instruction - JuMP to new location.
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirectX,
    IndirectIndexedY,
}

#[cfg(test)]
mod nes_test {
    use super::{Cpu, Nes, StatusFlag};
    use std::slice;
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

    #[test]
    fn load_to_rom_from_bytes_test() {
        let mut nes = Nes::default();

        // Check that the default memory is empty
        assert_eq!(nes.memory, [0; 0xFFFF]);

        // Simulation of game data
        const TEST_ROM_SIZE: usize = 0x0700;
        let test_rom = [0x08; TEST_ROM_SIZE];

        // Load catridge data to ROM
        // ROM is in the range 0x8000..0xFFFF
        nes.load_rom_from_bytes(&test_rom);

        // Check the range in memory to which data is being loaded
        assert_eq!(
            nes.memory[0x8000..0x8000 + TEST_ROM_SIZE],
            test_rom,
            "The data in the ROM was loaded incorrectly"
        );

        // Check the range that should have remained untouched
        assert_eq!(
            nes.memory[0..0x7FFF],
            [0; 0x7FFF],
            "The first 32 KiB should be empty"
        );
    }

    #[test]
    fn mem_write_read_8_test() {
        const ADDRESS: usize = 0x00FF;
        const VALUE: u8 = 0x1F;

        let mut nes = Nes::default();

        assert_eq!(nes.memory[ADDRESS], 0);

        nes.mem_write_8(ADDRESS as u16, VALUE);

        assert_eq!(nes.memory[ADDRESS], VALUE);
    }

    #[test]
    fn mem_read_8_test() {
        const ADDRESS: usize = 0x00FF;
        const VALUE: u8 = 0x1F;

        let mut nes = Nes::default();

        assert_eq!(nes.memory[ADDRESS], 0);

        nes.memory[ADDRESS] = VALUE;

        assert_eq!(nes.mem_read_8(ADDRESS as u16), VALUE);
    }

    #[test]
    fn mem_write_16() {
        const ADDRESS: usize = 0xFF1F;
        const VALUE: u16 = 0x7F1F;

        let mut nes = Nes::default();

        assert_eq!(nes.memory[ADDRESS], 0);
        assert_eq!(nes.memory[ADDRESS + 1], 0);

        nes.mem_write_16(ADDRESS as u16, VALUE);

        let [high, low] = VALUE.to_be_bytes();

        assert_eq!(nes.memory[ADDRESS], low);
        assert_eq!(nes.memory[ADDRESS + 1], high);
    }

    #[test]
    fn mem_read_16() {
        const ADDRESS: usize = 0x00FF;
        const VALUE_HIGH: u8 = 0x23;
        const VALUE_LOW: u8 = 0x1F;

        let mut nes = Nes::default();

        assert_eq!(nes.memory[ADDRESS], 0);
        assert_eq!(nes.memory[ADDRESS + 1], 0);

        // Little endian: the 8 least significant bits of an address will be stored
        // before the 8 most significant bits
        nes.memory[ADDRESS] = VALUE_LOW;
        nes.memory[ADDRESS + 1] = VALUE_HIGH;

        let data = nes.mem_read_16(ADDRESS as u16);
        let [low, high] = data.to_le_bytes();

        assert_eq!(high, VALUE_HIGH);
        assert_eq!(low, VALUE_LOW);
    }
}

#[cfg(test)]
mod addressing_mode_tests {
    use crate::cpu::Cpu;

    use super::{AddressingMode, Nes};

    #[test]
    fn addr_mode_accumulator_test() {
        todo!("Implement Accumulator addr. mode");
    }

    #[test]
    fn addr_mode_immediate_test() {
        let mut nes = Nes::default();
        let program_counter = 0xA080;

        nes.set_program_counter(program_counter);

        assert_eq!(
            nes.get_operand_address(AddressingMode::Immediate),
            program_counter
        );
    }

    #[test]
    fn addr_mode_absolute_test() {
        let mut nes = Nes::default();
        let program_counter = 0xA123;
        let expected_result = 0xF1;

        nes.set_program_counter(program_counter);
        nes.mem_write_8(program_counter, expected_result);

        assert_eq!(
            nes.get_operand_address(AddressingMode::Absolute),
            expected_result as u16
        );
    }

    #[test]
    fn addr_mode_zero_page_test() {
        let mut nes = Nes::default();
        let program_counter = 0x8001;
        let rom_data = 0x05;
        let expected_result = 0x43;

        nes.set_program_counter(program_counter);
        nes.mem_write_8(program_counter, rom_data);
        nes.mem_write_8(rom_data as u16, expected_result);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::ZeroPage));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn addr_mode_zero_page_x_test() {
        let register_x = 0x02;
        let cpu = Cpu::new(0x0, register_x, 0x0, 0x8001, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data = 0x05;
        let expected_result = 0x43;

        nes.mem_write_8(nes.cpu.program_counter, rom_data);
        nes.mem_write_8(rom_data.wrapping_add(register_x) as u16, expected_result);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::ZeroPageX));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn addr_mode_zero_page_y_test() {
        let register_y = 0x04;
        let cpu = Cpu::new(0x0, 0x0, register_y, 0x8001, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data = 0x05;
        let expected_result = 0x43;

        nes.mem_write_8(nes.cpu.program_counter, rom_data);
        nes.mem_write_8(rom_data.wrapping_add(register_y) as u16, expected_result);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::ZeroPageY));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn addr_mode_absolute_x_test() {
        let register_x = 0x01;
        let cpu = Cpu::new(0x0, register_x, 0x0, 0x8001, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data: u16 = 0x0200;
        let expected_result = 0x43;

        nes.mem_write_16(nes.cpu.program_counter, rom_data);
        nes.mem_write_8(rom_data.wrapping_add(register_x as u16), expected_result);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::AbsoluteX));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn addr_mode_absolute_y_test() {
        let register_y = 0x04;
        let cpu = Cpu::new(0x0, 0x0, register_y, 0x8001, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data: u16 = 0x0200;
        let expected_resukt = 0x43;

        nes.mem_write_16(nes.cpu.program_counter, rom_data);
        nes.mem_write_8(rom_data.wrapping_add(register_y as u16), expected_resukt);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::AbsoluteY));

        assert_eq!(result, expected_resukt);
    }

    #[test]
    fn addr_mode_indirect_test() {
        todo!("Implement Indirect addr. mode");
    }

    #[test]
    fn addr_mode_indexed_indirect_x_test() {
        let register_x = 0x01;
        let program_counter = 0x8001;
        let cpu = Cpu::new(0x0, register_x, 0x0, program_counter, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data = 0x05;
        let stored_address = 0x0705;
        let expected_result = 0x1A;

        nes.mem_write_8(program_counter, rom_data);
        nes.mem_write_16(
            (rom_data as u16).wrapping_add(register_x as u16),
            stored_address,
        );
        nes.mem_write_8(stored_address, expected_result);

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::IndexedIndirectX));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn addr_mode_indirect_indexed_y_test() {
        let register_y = 0x02;
        let program_counter = 0x8001;
        let cpu = Cpu::new(0x0, 0x0, register_y, program_counter, 0x0, 0x0);
        let mut nes = Nes::new(cpu);
        let rom_data = 0x05;
        let stored_address = 0x0703;
        let expected_result = 0x1A;

        nes.mem_write_8(program_counter, rom_data);
        nes.mem_write_16(rom_data as u16, stored_address);
        nes.mem_write_8(
            stored_address.wrapping_add(register_y as u16),
            expected_result,
        );

        let result = nes.mem_read_8(nes.get_operand_address(AddressingMode::IndirectIndexedY));

        assert_eq!(result, expected_result);
    }
}
