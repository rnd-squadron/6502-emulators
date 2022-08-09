// TODO: Remove this lint rules
#![allow(unused)]

use std::{fs, io::Read, slice};

use strum_macros::EnumIter;

pub struct Nes {
    pub cpu: Cpu,
    pub memory: [u8; 0xFFFF], // 64 Kib
}

impl Default for Nes { 
    fn default() -> Self {
        Nes { 
            cpu: Cpu::default(),
            memory: [0; 0xFFFF]
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

    pub fn load(&mut self, data: [u8; 0xFFFF]) {
        self.memory = data;
    }

    pub fn load_rom_from_bytes(&mut self, data: &[u8]) {
        self.memory[0x8001..0x8001 + data.len()].copy_from_slice(data);
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
        let high = self.mem_read_8(address + 1) as u16;

        (high << 8) | low
    }

    pub fn mem_write_16(&mut self, address: u16, data: u16) {
        let [high, low] = [(data >> 8) as u8, (data & 0xFF) as u8];

        self.mem_write_8(address, low);
        self.mem_write_8(address + 1, high);
    }
}

#[derive(Default)]
pub struct Cpu {
    accumulator: u8,
    register_x: u8,
    register_y: u8,
    program_counter: u16,
    status: u8,
    stack_pointer: u8,
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
            nes.memory[0x8001..0x8001 + TEST_ROM_SIZE],
            test_rom,
            "The data in the ROM was loaded incorrectly"
        );

        // Check the range that should have remained untouched
        assert_eq!(
            nes.memory[0..0x8000],
            [0; 0x8000],
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

        nes.memory[ADDRESS] = VALUE_HIGH;
        nes.memory[ADDRESS + 1] = VALUE_LOW;

        let data = nes.mem_read_16(ADDRESS as u16);
        let (low, high) = ((data >> 8) as u8, (data & 0xFF) as u8);

        assert_eq!(high, VALUE_HIGH);
        assert_eq!(low, VALUE_LOW);
    }
}
