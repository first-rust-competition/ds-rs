pub mod tags;


/// bitflag struct for the Control value of the packet
bitflags! {
    pub struct Control: u8 {
        const ESTOP = 0b1000_0000;
        const FMS_CONNECTED = 0b0000_1000;
        const ENABLED = 0b0000_0100;

        // Mode flags
        const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
    }
}

/// Trait/structs for reboot and code restart requests
///
/// Not contained in a bitflags struct because these reqeusts are exclusive (can't/shouldn't be OR'd)
pub trait Request {
    fn code(&self) -> u8;
}

pub struct RebootRoborio;

impl Request for RebootRoborio {
    fn code(&self) -> u8 {
        0b0000_1000
    }
}

pub struct RestartCode;

impl Request for RestartCode {
    fn code(&self) -> u8 {
        0b0000_0100
    }
}

#[derive(Copy, Clone)]
/// Struct abstracting the byte value for alliance colour and position
pub struct Alliance(pub u8);

impl Alliance {
    pub fn new_red(position: u8) -> Alliance {
//        assert!((1u8..3).contains(&position));

        Alliance(position - 1)
    }

    pub fn new_blue(position: u8) -> Alliance {
//        assert!((1u8..3).contains(&position));

        Alliance(position + 2)
    }

    pub fn is_red(self) -> bool {
        self.0 < 3
    }

    pub fn is_blue(self) -> bool {
        !self.is_red()
    }

    pub fn position(self) -> u8 {
        (self.0 % 3) + 1
    }
}