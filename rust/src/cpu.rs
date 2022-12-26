// TODO: Remove this lint rules
#![allow(unused)]

use std::{cmp, fs, io::Read, ops::Deref, slice};
use strum_macros::EnumIter;

use crate::instructions::{Instruction, OpCode};

const STACK_START: u16 = 0x0100;

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
        self.cpu.stack_pointer = 0xFD;

        // Reset vector: read from $FFFC and $FFFD
        self.cpu.program_counter = self.mem_read_16(0xFFFC);
    }

    pub fn set_program_counter(&mut self, address: u16) {
        self.cpu.program_counter = address;
    }

    pub fn load(&mut self, data: [u8; 0xFFFF]) {
        self.memory = data;
    }

    pub fn load_instructions(&mut self, program_mem: Vec<u8>) {
        program_mem.iter().enumerate().for_each(|(index, &code)| {
            self.mem_write_8(0x0600 + index as u16, code);
        })
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

    pub fn pop_stack(&mut self) -> u8 {
        self.cpu.stack_pointer = self.cpu.stack_pointer.wrapping_add(1);
        self.mem_read_8(STACK_START + self.cpu.stack_pointer as u16)
    }

    pub fn push_stack(&mut self, data: u8) {
        self.mem_write_8(STACK_START + self.cpu.stack_pointer as u16, data);
        self.cpu.stack_pointer = self.cpu.stack_pointer.wrapping_sub(1);
    }

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
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
            _ => panic!("Addressing mode not implemented!"),
        }
    }

    pub fn run_with_reset_pc(&mut self, reset_program_counter: bool) {
        self.reset();

        if reset_program_counter {
            self.cpu.program_counter = 0x0600;
        }

        self.run()
    }

    fn run(&mut self) {
        // Main loop
        loop {
            let code = self.mem_read_8(self.cpu.program_counter);

            self.cpu.program_counter += 1;

            let current_pc = self.cpu.program_counter;
            let opcode = OpCode::from_byte(code);

            match (&opcode.instruction, code) {
                // Stop code
                (Instruction::Brk, _) => return,
                // ADC
                (Instruction::Adc, _) => todo!("Implement ADC instruction"),
                // AND
                (Instruction::And, _) => self.and(&opcode),
                // ASL
                (Instruction::Asl, _) => self.asl(&opcode),
                // CMP
                (Instruction::Cmp, _) => self.cmp(&opcode),
                // CPX
                (Instruction::Cpx, _) => self.cpx(&opcode),
                // CPY
                (Instruction::Cpy, _) => self.cpy(&opcode),
                // DEC
                (Instruction::Dec, _) => self.dec(&opcode),
                // EOR
                (Instruction::Eor, _) => self.eor(&opcode),
                // INC
                (Instruction::Inc, _) => self.inc(&opcode),
                // JMP
                (Instruction::Jmp, _) => self.jmp(&opcode),
                // JSR
                (Instruction::Jsr, _) => todo!("Implement JSR instruction"),
                // LDA
                (Instruction::Lda, _) => self.lda(&opcode),
                // LDX
                (Instruction::Ldx, _) => self.ldx(&opcode),
                // LDY
                (Instruction::Ldy, _) => self.ldy(&opcode),
                // LSR
                (Instruction::Lsr, _) => self.lsr(&opcode),
                // ORA
                (Instruction::Ora, _) => self.ora(&opcode),
                // ROL
                (Instruction::Rol, _) => self.rol(&opcode),
                // ROR
                (Instruction::Ror, _) => self.ror(&opcode),
                // SBC
                (Instruction::Sbc, _) => todo!("Implement SBC instruction"),
                // STA
                (Instruction::Sta, _) => self.sta(&opcode),
                // STX
                (Instruction::Stx, _) => self.stx(&opcode),
                // STY
                (Instruction::Sty, _) => self.sty(&opcode),
                // SEC
                (Instruction::Sec, _) => self.sec(),
                // SED
                (Instruction::Sed, _) => self.sed(),
                // SEI
                (Instruction::Sei, _) => self.sei(),
                // CLC
                (Instruction::Clc, _) => self.clc(),
                // CLD
                (Instruction::Cld, _) => self.cld(),
                // CLI
                (Instruction::Cli, _) => self.cli(),
                // CLV
                (Instruction::Clv, _) => self.clv(),
                // BMI
                (Instruction::Bmi, _) => self.bmi(&opcode),
                // BPL
                (Instruction::Bpl, _) => self.bpl(&opcode),
                // BVS
                (Instruction::Bvs, _) => self.bvs(&opcode),
                // BVC
                (Instruction::Bvc, _) => self.bvc(&opcode),
                // BCS
                (Instruction::Bcs, _) => self.bcs(&opcode),
                // BCC
                (Instruction::Bcc, _) => self.bcc(&opcode),
                // BEQ
                (Instruction::Beq, _) => self.beq(&opcode),
                // BNE
                (Instruction::Bne, _) => self.bne(&opcode),
                // Other
                _ => todo!("Code: {:x?} not implemented!", code),
            };

            self.update_pc(current_pc, opcode.bytes);
        }
    }

    fn update_pc(&mut self, current_pc: u16, bytes: u8) {
        if current_pc == self.cpu.program_counter {
            self.cpu.program_counter += (bytes - 1) as u16;
        }
    }

    //Operations for transferring bytes of data
    fn lda(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn ldx(&mut self, opcode: &OpCode) {
        let adderss = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(adderss);

        self.cpu.register_x = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn ldy(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.register_y = value;
        self.cpu.update_zero_and_negative_flags(value);
    }

    fn sta(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);

        self.mem_write_8(address, self.cpu.accumulator);
    }

    fn stx(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);

        self.mem_write_8(address, self.cpu.register_x)
    }

    fn sty(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);

        self.mem_write_8(address, self.cpu.register_y)
    }

    fn tax(&mut self, opcode: &OpCode) {
        self.cpu.register_x = self.cpu.accumulator;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_x);
    }

    fn tay(&mut self, opcode: &OpCode) {
        self.cpu.register_y = self.cpu.accumulator;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_y);
    }

    fn txa(&mut self, opcode: &OpCode) {
        self.cpu.accumulator = self.cpu.register_x;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn tya(&mut self, opcode: &OpCode) {
        self.cpu.accumulator = self.cpu.register_y;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn txs(&mut self, opcode: &OpCode) {
        self.cpu.stack_pointer = self.cpu.register_x;
    }

    fn tsx(&mut self, opcode: &OpCode) {
        self.cpu.register_x = self.cpu.stack_pointer;
        self.cpu.update_zero_and_negative_flags(self.cpu.register_x);
    }

    // Addition
    fn adc(&mut self, opcode: &OpCode) {}

    // Subtraction
    fn sub(&mut self, opcode: &OpCode) {}

    // Bitwise operations
    fn and(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator &= value;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn ora(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator |= value;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    fn eor(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        self.cpu.accumulator ^= value;
        self.cpu
            .update_zero_and_negative_flags(self.cpu.accumulator);
    }

    // Operations for incrementing and decrementing the index registers
    fn inx(&mut self, opcode: &OpCode) {
        let (result, _) = self.cpu.register_x.overflowing_add(1);

        self.cpu.register_x = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn iny(&mut self, opcode: &OpCode) {
        let (result, _) = self.cpu.register_y.overflowing_add(1);

        self.cpu.register_y = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dex(&mut self, opcode: &OpCode) {
        let (result, _) = self.cpu.register_x.overflowing_sub(1);

        self.cpu.register_x = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dey(&mut self, opcode: &OpCode) {
        let (result, _) = self.cpu.register_y.overflowing_sub(1);

        self.cpu.register_y = result;
        self.cpu.update_zero_and_negative_flags(result);
    }

    // Operations for incrementing and decrementing memory
    fn inc(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let (result, _) = value.overflowing_add(1);

        self.mem_write_8(address, result);
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn dec(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let (result, _) = value.overflowing_sub(1);

        self.mem_write_8(address, result);
        self.cpu.update_zero_and_negative_flags(result);
    }

    // Operations for byte comparison
    fn cmp(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.accumulator.wrapping_sub(value);

        self.cpu
            .update_flag(&StatusFlag::Carry, self.cpu.accumulator >= value);
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn cpx(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.register_x.wrapping_sub(value);

        self.cpu
            .update_flag(&StatusFlag::Carry, self.cpu.register_x >= value);
        self.cpu.update_zero_and_negative_flags(result);
    }

    fn cpy(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.register_y.wrapping_sub(value);

        self.cpu
            .update_flag(&StatusFlag::Carry, self.cpu.register_y >= value);
        self.cpu.update_zero_and_negative_flags(result);
    }

    // The BIT operation
    fn bit(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);
        let result = self.cpu.accumulator & value;

        self.cpu.update_flag(&StatusFlag::Overflow, value >> 6 == 1);
        self.cpu.update_zero_and_negative_flags(result);
    }

    // Bit shift operations
    fn lsr(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        let (result, _) = value.overflowing_shr(1);

        self.mem_write_8(address, result);

        self.cpu.update_flag(&StatusFlag::Zero, value >> 7 == 0);
        self.cpu.update_flag(&StatusFlag::Negative, false);
        self.cpu.update_flag(&StatusFlag::Carry, value & 1 == 1);
    }

    fn asl(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        let (result, _) = value.overflowing_shl(1);

        self.mem_write_8(address, result);

        self.cpu.update_flag(&StatusFlag::Zero, value >> 7 == 0);
        self.cpu
            .update_flag(&StatusFlag::Negative, result >> 7 == 1);
        self.cpu.update_flag(&StatusFlag::Carry, value >> 7 == 1);
    }

    fn ror(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        let mut result = value.rotate_right(1);

        self.cpu.update_flag(&StatusFlag::Carry, value >> 7 == 1);

        if self.cpu.has_flag(&StatusFlag::Carry) {
            result |= 0b10000000;
        }

        self.cpu.update_flag(&StatusFlag::Zero, value >> 7 == 0);
        self.cpu
            .update_flag(&StatusFlag::Negative, result >> 7 == 1);
    }

    fn rol(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_8(address);

        let mut result = value.rotate_left(1);

        self.cpu.update_flag(&StatusFlag::Carry, value >> 7 == 1);

        if self.cpu.has_flag(&StatusFlag::Carry) {
            result |= 1;
        }

        self.cpu.update_flag(&StatusFlag::Zero, value >> 7 == 0);
        self.cpu
            .update_flag(&StatusFlag::Negative, result >> 7 == 1);
    }

    // The Jump operation
    fn jmp(&mut self, opcode: &OpCode) {
        let address = self.get_operand_address(&opcode.address_mode);
        let value = self.mem_read_16(address);

        self.cpu.program_counter = value;
    }

    // Operations for setting and clearing the Processor Status register flags
    fn sec(&mut self) {
        self.cpu.enable_flag(&StatusFlag::Carry);
    }

    fn sed(&mut self) {
        self.cpu.enable_flag(&StatusFlag::Decimal);
    }

    fn sei(&mut self) {
        self.cpu.enable_flag(&StatusFlag::Interrupt);
    }

    fn clc(&mut self) {
        self.cpu.disable_flag(&StatusFlag::Carry);
    }

    fn cld(&mut self) {
        self.cpu.disable_flag(&StatusFlag::Decimal);
    }

    fn cli(&mut self) {
        self.cpu.disable_flag(&StatusFlag::Interrupt);
    }

    fn clv(&mut self) {
        self.cpu.disable_flag(&StatusFlag::Overflow);
    }

    fn bmi(&mut self, opcode: &OpCode) {
        if self.cpu.has_flag(&StatusFlag::Negative) {
            self.jmp(opcode)
        }
    }

    fn bpl(&mut self, opcode: &OpCode) {
        if !self.cpu.has_flag(&StatusFlag::Negative) {
            self.jmp(opcode)
        }
    }

    fn bvs(&mut self, opcode: &OpCode) {
        if self.cpu.has_flag(&StatusFlag::Overflow) {
            self.jmp(opcode)
        }
    }

    fn bvc(&mut self, opcode: &OpCode) {
        if !self.cpu.has_flag(&StatusFlag::Overflow) {
            self.jmp(opcode)
        }
    }

    fn bcs(&mut self, opcode: &OpCode) {
        if self.cpu.has_flag(&StatusFlag::Carry) {
            self.jmp(opcode)
        }
    }

    fn bcc(&mut self, opcode: &OpCode) {
        if !self.cpu.has_flag(&StatusFlag::Carry) {
            self.jmp(opcode)
        }
    }

    fn beq(&mut self, opcode: &OpCode) {
        if self.cpu.has_flag(&StatusFlag::Zero) {
            self.jmp(opcode)
        }
    }

    fn bne(&mut self, opcode: &OpCode) {
        if !self.cpu.has_flag(&StatusFlag::Zero) {
            self.jmp(opcode)
        }
    }

    //
}

#[derive(Debug)]
pub struct Cpu {
    pub accumulator: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub status: u8,
    pub stack_pointer: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0x0600,
            status: 0b00100100,
            stack_pointer: 0xfd,
        }
    }
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
    Implied,
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
            cpu.status = 0b00000000;

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
    use crate::{cpu::Cpu, instructions::OpCode};

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
            nes.get_operand_address(&AddressingMode::Immediate),
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
            nes.get_operand_address(&AddressingMode::Absolute),
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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::ZeroPage));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::ZeroPageX));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::ZeroPageY));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::AbsoluteX));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::AbsoluteY));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::IndexedIndirectX));

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

        let result = nes.mem_read_8(nes.get_operand_address(&AddressingMode::IndirectIndexedY));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn lda_immediate_test() {
        let mut nes = Nes::default();

        nes.mem_write_8(nes.cpu.program_counter + 1, 0x80);

        nes.load_instructions(vec![0xA9]);
        nes.run_with_reset_pc(true);

        assert_eq!(nes.cpu.accumulator, 0x80);
    }

    #[test]
    fn load_to_and_store_to_zero_page_test() {
        let mut nes = Nes::default();

        // 0080: F1, F2, F3, 00,  00
        nes.mem_write_8(0x80, 0xF1);
        nes.mem_write_8(0x81, 0xF2);
        nes.mem_write_8(0x82, 0xF3);

        nes.load_instructions(vec![
            0xA5, 0x80, // LDA $80
            0x85, 0x20, // STA $20
            0xA6, 0x81, // LDX $81
            0x86, 0x21, // STX $21
            0xA4, 0x82, // LDY $82
            0x84, 0x22, // STY $22
        ]);

        nes.run_with_reset_pc(true);

        let sta_result = nes.mem_read_8(0x20);
        let stx_result = nes.mem_read_8(0x21);
        let sty_result = nes.mem_read_8(0x22);

        assert_eq!(sta_result, 0xF1);
        assert_eq!(stx_result, 0xF2);
        assert_eq!(sty_result, 0xF3);
    }
}
