use libds;
use libc::{c_int, c_char, c_uint, c_void, c_float};
use std::ffi::CStr;

use std::ptr;

use crate::alliance::Alliance;

#[repr(C)]
pub enum Mode {
    Autonomous,
    Teleoperated,
    Test
}

impl Mode {
    fn to_rust_mode(self) -> libds::Mode {
        match self {
            Mode::Autonomous => libds::Mode::Autonomous,
            Mode::Teleoperated => libds::Mode::Teleoperated,
            Mode::Test => libds::Mode::Test,
        }
    }
}

#[repr(C)]
pub struct DriverStation {
    inner: *mut c_void
}

macro_rules! assert_nonnull {
    ($ptr:expr $(, $ret:expr)?) => {
        if $ptr.is_null() {
            return $($ret)?;
        }
    }
}

#[no_mangle]
pub extern "C" fn DriverStation_new(team_number: c_uint, alliance: *mut Alliance) -> *mut DriverStation {
    assert_nonnull!(alliance, ptr::null_mut());

    let alliance = unsafe { Box::from_raw(alliance).inner as *mut libds::Alliance };
    Box::into_raw(Box::new(DriverStation {
        inner: Box::into_raw(Box::new(libds::DriverStation::new(unsafe { *Box::from_raw(alliance) }, team_number))) as *mut c_void
    }))
}

#[no_mangle]
pub extern "C" fn DriverStation_connected(ptr: *mut DriverStation) -> c_int {
    assert_nonnull!(ptr, 0);

    let ds = unsafe { &*((*ptr).inner as *mut libds::DriverStation) };

    match ds.connected() {
        Ok(true) => 1,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn DriverStation_set_mode(ptr: *mut DriverStation, mode: Mode) {
    assert_nonnull!(ptr);

    let ds = unsafe { &mut *((*ptr).inner as *mut libds::DriverStation) };
    ds.set_mode(mode.to_rust_mode());
}

#[no_mangle]
pub extern "C" fn DriverStation_set_game_specific_message(ptr: *mut DriverStation, gsm: *const c_char) {
    assert_nonnull!(ptr);
    assert_nonnull!(gsm);

    let s = unsafe {
        CStr::from_ptr(gsm)
    };

    let ds = unsafe { &mut *((*ptr).inner as *mut libds::DriverStation) };

    ds.set_game_specific_message(&s.to_string_lossy());
}

#[no_mangle]
pub extern "C" fn DriverStation_enable(ptr: *mut DriverStation) {
    assert_nonnull!(ptr);

    let ds = unsafe { &mut *((*ptr).inner as *mut libds::DriverStation) };
    ds.enable();
}

#[no_mangle]
pub extern "C" fn DriverStation_disable(ptr: *mut DriverStation) {
    assert_nonnull!(ptr);

    let ds = unsafe { &mut *((*ptr).inner as *mut libds::DriverStation) };
    ds.disable();
}

#[no_mangle]
pub extern "C" fn DriverStation_estop(ptr: *mut DriverStation) {
    assert_nonnull!(ptr);

    let ds = unsafe { &mut *((*ptr).inner as *mut libds::DriverStation) };
    ds.estop();
}

#[no_mangle]
pub extern "C" fn DriverStation_get_battery_voltage(ptr: *mut DriverStation) -> c_float {
    assert_nonnull!(ptr, 0f32);

    let ds = unsafe { &*((*ptr).inner as *mut libds::DriverStation) };
    ds.battery_voltage()
}

#[no_mangle]
pub extern "C" fn DriverStation_free(ptr: *mut DriverStation) {
    assert_nonnull!(ptr);

    let cds = unsafe { Box::from_raw(ptr) };
    unsafe { Box::from_raw(cds.inner as *mut DriverStation); }
}