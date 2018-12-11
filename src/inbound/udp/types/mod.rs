pub mod tag;

bitflags! {
    pub struct Status: u8 {
        const ESTOP = 0b1000_0000;
        const BROWNOUT = 0b0001_0000;
        const CODE_START = 0b0000_1000;
        const ENABLED = 0b0000_0100;

        // Mode flags
        const TELEOP = 0b00;
        const TEST = 0b01;
        const AUTO = 0b10;
    }
}

impl Status {
    pub fn is_browning_out(self) -> bool {
        self & Status::BROWNOUT == Status::BROWNOUT
    }
}

bitflags! {
    pub struct Trace: u8 {
        const ROBOT_CODE = 0b0010_0000;
        const IS_ROBORIO = 0b0001_0000;
        const TEST_MODE = 0b0000_1000;
        const AUTONOMOUS = 0b0000_0100;
        const TELEOP = 0b0000_0010;
        const DISABLED = 0b0000_0001;
    }
}

macro_rules! gen_trace_methods {
    ($($func_name:ident => $flag_name:expr),+) => {
        impl Trace {
            $(
            pub fn $func_name(self) -> bool {
                self & $flag_name == $flag_name
            }
            )+
        }
    }
}

gen_trace_methods!(is_autonomous => Trace::AUTONOMOUS, is_teleop => Trace::TELEOP, is_disabled => Trace::DISABLED,
                   is_test => Trace::TEST_MODE, is_code_started => Trace::ROBOT_CODE, is_connected => Trace::IS_ROBORIO);
