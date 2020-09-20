use std::ffi::CString;
use std::marker::PhantomData;

mod cds;
mod trace;
mod joysticks;

#[repr(C)]
pub struct StdoutMessage<'a> {
    message: *const libc::c_char,
    _lifetime: PhantomData<&'a CString>,
}

impl<'a> StdoutMessage<'a> {
    pub(crate) fn new(message: &'a CString) -> StdoutMessage {
        StdoutMessage {
            message: message.as_ptr(),
            _lifetime: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum Mode {
    Autonomous,
    Teleoperated,
    Test
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum DsMode {
    Normal,
    Simulation
}

impl Mode {
    pub(crate) fn to_ds(self) -> ds::Mode {
        match self {
            Mode::Autonomous => ds::Mode::Autonomous,
            Mode::Teleoperated => ds::Mode::Teleoperated,
            Mode::Test => ds::Mode::Test,
        }
    }

    pub(crate) fn from_ds(ds: ds::Mode) -> Mode {
        match ds {
            ds::Mode::Autonomous => Mode::Autonomous,
            ds::Mode::Teleoperated => Mode::Teleoperated,
            ds::Mode::Test => Mode::Test,
        }
    }
}

impl DsMode {
    pub(crate) fn from_ds(ds: ds::DsMode) -> DsMode {
        match ds {
            ds::DsMode::Normal => DsMode::Normal,
            ds::DsMode::Simulation => DsMode::Simulation
        }
    }
}