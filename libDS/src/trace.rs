//! Raw bit values for the Trace type

/// The mask for robot code being alive
pub const TRACE_ROBOT_CODE: u8 = 0b0010_0000;
/// The mask for the target being a roboRIO
pub const TRACE_IS_ROBORIO: u8 = 0b0001_0000;
/// The mask for Test mode being selected
pub const TRACE_TEST_MODE: u8 = 0b0000_1000;
/// The mask for Autonomous mode being selected
pub const TRACE_AUTONOMOUS: u8 = 0b0000_0100;
/// The mask for Teleop mode being selected
pub const TRACE_TELEOP: u8 = 0b0000_0010;
/// The mask for the robot being disabled
pub const TRACE_DISABLED: u8 = 0b0000_0001;