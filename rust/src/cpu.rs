// TODO: Remove this lint rules
#![allow(unused)]

use std::{fs, io::Read, slice};

use strum_macros::EnumIter;

pub struct Nes {
    pub cpu: Cpu,
    pub memory: [u8; 0xFFFF], // 64 Kib
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

    pub fn mem_write_8(&mut self, data: u8, address: u16) {
        self.memory[address as usize] = data;
    }

    pub fn mem_read_16(&self, address: u16) -> u16 {
        let low = self.mem_read_8(address) as u16;
        let high = self.mem_read_8(address + 1) as u16;

        (high << 8) | low
    }

    pub fn mem_write_16(&mut self, address: u16, data: u16) {
        let low = (data >> 8) as u8;
        let high = (data & 0xff) as u8;

        self.mem_write_8(low, address);
        self.mem_write_8(high + 1, address);
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

        // self.program_counter = self.
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

#[allow(clippy::upper_case_acronyms)]
pub struct Rom {
    cartridge_rom: Vec<u8>,
}

impl Rom {
    fn from_file(filename: String) -> Self {
        let file = fs::File::open(&filename).expect("File not found");

        file.bytes()
            .take(32_767)
            .collect::<Result<Vec<u8>, _>>()
            .map(Rom::from_bytes)
            .expect("Error processing byte stream for ROM")
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Rom {
            cartridge_rom: bytes,
        }
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
        let cpu = Cpu::default();
        let mut nes = Nes::new(cpu);

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
    fn mem_write_8_test() {
        unimplemented!()
    }

    #[test]
    fn mem_read_8_test() {
        unimplemented!()
    }

    #[test]
    fn mem_write_16() {
        unimplemented!()
    }

    #[test]
    fn mem_read_16() {
        unimplemented!()
    }
}
