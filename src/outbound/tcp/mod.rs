pub mod tags;

pub struct TcpControlPacket {
    id: u8,

}

#[repr(u8)]
pub enum AxisType {
    XAxis = 0,
    YAxis = 1,
    ZAxis = 2,
    TwistAxis = 3,
    ThrottleAxis = 4,
}

#[repr(i8)]
pub enum JoystickType {
    Unknown = -1,
    XinputUnknown = 0,
    XinputGamepad = 1,
    XinputWheel = 2,
    XinputArcade = 3,
    XinputFlightStick = 4,
    XinputDancePad = 5,
    XinputGuitar = 6,
    XinputGuitar2 = 7,
    XinputDrumKit = 8,
    XinputGuitar3 = 11,
    XinputAracadePad = 19,
    HIDJoystick = 20,
    HIDGamepad = 21,
    HIDDriving = 22,
    HIDFlight = 23,
    HID1stPerson = 24,
}
