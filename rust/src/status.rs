use std::fmt::Debug;

#[derive(Debug)]
enum Status {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Constant,
    Overflow,
    Negative,
}

impl Status {
    fn bit_shift(&self) -> u8 {
        match self {
            Status::Carry => 0x01,
            Status::Zero => 0x02,
            Status::Interrupt => 0x04,
            Status::Decimal => 0x08,
            Status::Break => 0x10,
            Status::Constant => 0x20,
            Status::Overflow => 0x40,
            Status::Negative => 0x80,
        }
    }

    fn has_flag(&self, value: u8) -> bool {
        (value & self.bit_shift()) != 0
    }
}

#[test]
fn status_test() {
    let data = [
        (Status::Carry, 0b00000001, 0b11111110),
        (Status::Zero, 0b00000010, 0b11111101),
        (Status::Interrupt, 0b00000000, 0b11111011),
        (Status::Decimal, 0b00001000, 0b11110111),
        (Status::Break, 0b00010000, 0b11101111),
        (Status::Constant, 0b00100000, 0b11011111),
        (Status::Overflow, 0b01000000, 0b10111111),
        (Status::Negative, 0b10000000, 0b01111111),
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
