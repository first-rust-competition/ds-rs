pub mod tags;

bitflags! {
    pub struct Control: u8 {
        const ESTOP = 0b10000000;
        const FMS_CONNECTED = 0b00001000;
        const ENABLED = 0b00000100;

        // Mode flags
        const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
    }
}

pub trait Request {
    fn code(&self) -> u8;
}

pub struct RebootRoborio;

impl Request for RebootRoborio {
    fn code(&self) -> u8 {
        0b00001000
    }
}

pub struct RestartCode;

impl Request for RestartCode {
    fn code(&self) -> u8 {
        0b00000100
    }
}

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

    pub fn is_red(&self) -> bool {
        self.0 < 3
    }

    pub fn position(&self) -> u8 {
        (self.0 % 3) + 1
    }
}