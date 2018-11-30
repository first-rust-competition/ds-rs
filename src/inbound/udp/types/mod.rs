pub mod tag;

use bitflags::bitflags;

bitflags! {
    pub struct Status: u8 {
        const ESTOP = 0b10000000;
        const BROWNOUT = 0b00010000;
        const CODE_START = 0b00001000;
        const ENABLED = 0b00000100;

        // Mode flags
        const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
    }
}

bitflags! {
    pub struct Trace: u8 {
        const ROBOT_CODE = 0b00100000;
        const IS_ROBORIO = 0b00010000;
        const TEST_MODE = 0b00001000;
        const AUTONOMOUS = 0b00000100;
        const TELEOP = 0b00000010;
        const DISABLED = 0b00000001;
    }
}