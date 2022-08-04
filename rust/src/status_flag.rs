use std::fmt::Debug;

#[derive(Debug)]
enum StatusFlag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Constant,
    Overflow,
    Negative,
}

impl StatusFlag{
    fn bit_shift(&self) -> u8 {
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

    fn has_flag(&self, value: u8) -> bool {
        (value & self.bit_shift()) != 0
    }
}

fn enable_flag(status: u8, flag: StatusFlag) -> u8 {
    status | flag.bit_shift()
}

fn disable_flag(status: u8, flag: StatusFlag) -> u8 { 
    status ^ flag.bit_shift()
}
  
#[test]
fn status_test() {
    let data = [
        (StatusFlag::Carry, 0b00000001, 0b11111110),
        (StatusFlag::Zero, 0b00000010, 0b11111101),
        (StatusFlag::Interrupt, 0b00000000, 0b11111011),
        (StatusFlag::Decimal, 0b00001000, 0b11110111),
        (StatusFlag::Break, 0b00010000, 0b11101111),
        (StatusFlag::Constant, 0b00100000, 0b11011111),
        (StatusFlag::Overflow, 0b01000000, 0b10111111),
        (StatusFlag::Negative, 0b10000000, 0b01111111),
    ];

    for case in data {
        let (status, flag_on, flag_off) = case;

        assert!(
            status.has_flag(flag_on),
            "Status {:?} must not have a flag",
            flag_on
        );
        assert!(
            !status.has_flag(flag_off),
            "Status {:?} must have a flag",
            flag_off
        );

    }
}
