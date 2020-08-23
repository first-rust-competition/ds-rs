use ds::{Alliance, DriverStation, TcpPacket};
use std::ptr;
use libc::c_char;
use std::ffi::{CStr, CString};
use crate::{Mode, DsMode, StdoutMessage};

/// Constructs a new Alliance representing a Red alliance robot of the given position
#[no_mangle]
pub extern "C" fn DS_Alliance_new_red(position: u8) -> *mut Alliance {
    let ptr = Box::new(Alliance::new_red(position));

    Box::into_raw(ptr)
}

/// Constructs a new Alliance representing a Blue alliance robot of the given position
#[no_mangle]
pub extern "C" fn DS_Alliance_new_blue(position: u8) -> *mut Alliance {
    let ptr = Box::new(Alliance::new_blue(position));

    Box::into_raw(ptr)
}

/// Constructs a new DriverStation that will connect to 10.TE.AM.2 with the given team, and that will be assigned the given alliance.
///
/// This function will return NULL if alliance is NULL
/// After calling this function, alliance will no longer be a valid pointer. Attempting to use it may result in UB.
/// The pointer returned by this function **must** be freed using DS_DriverStation_destroy(). Using any other means is undefined.
#[no_mangle]
pub extern "C" fn DS_DriverStation_new_team(team_number: u32, alliance: *mut Alliance) -> *mut DriverStation {
    if alliance.is_null() {
        return ptr::null_mut();
    }

    let alliance = unsafe { Box::from_raw(alliance) };
    let ds = Box::new(DriverStation::new_team(team_number, *alliance));

    Box::into_raw(ds)
}

/// Constructs a new DriverStation that will connect to the specified IP, and that will be assigned the given alliance and team number
///
/// This function will return NULL if alliance or ip is NULL
/// After calling this function, alliance will no longer be a valid pointer. Attempting to use it may result in UB.
/// The pointer returned by this function **must** be freed using DS_DriverStation_destroy(). Using any other means is undefined.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_new_ip(ip: *const c_char, alliance: *mut Alliance, team_number: u32) -> *mut DriverStation {
    if ip.is_null() || alliance.is_null() {
        return ptr::null_mut();
    }

    let ip = CStr::from_ptr(ip);

    let alliance = Box::from_raw(alliance);
    let ds = Box::new(DriverStation::new(ip.to_str().unwrap(), *alliance, team_number));

    Box::into_raw(ds)
}

/// Safely frees a given DriverStation.
///
/// This function should only be passed pointers that were allocated via DS_DriverStation_new_team or DS_DriverStation_new_ip
#[no_mangle]
pub extern "C" fn DS_DriverStation_destroy(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    unsafe { Box::from_raw(ds); }
}

/// Assigns the given alliance station to the given driver station
///
/// This function does nothing if ds or alliance are NULL
/// After calling this function, the alliance pointer will no longer be valid.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_alliance(ds: *mut DriverStation, alliance: *mut Alliance) {
    if ds.is_null() || alliance.is_null() {
        return;
    }

    (*ds).set_alliance(*Box::from_raw(alliance));
}

/// Updates the team number of the given driver station. This will automatically reconnect the
/// network threads to target 10.TE.AM.2
///
/// This function does nothing if ds is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_team_number(ds: *mut DriverStation, team_number: u32) {
    if ds.is_null() {
        return;
    }

    (*ds).set_team_number(team_number);
}

/// Specifies whether the driver station should attempt to connect to 172.22.11.2 over USB rather than any other specified target
///
/// This function does nothing if ds is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_use_usb(ds: *mut DriverStation, use_usb: bool) {
    if ds.is_null() {
        return;
    }

    (*ds).set_use_usb(use_usb);
}

/// Gets the team number currently assigned to the given DriverStation
///
/// This function will return 0 if the given ds is NULL.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_get_team_number(ds: *const DriverStation) -> u32 {
    if ds.is_null() {
        return 0;
    }

    (*ds).team_number()
}

/// Updates the Game Specific Message (GSM) associated with the given DriverStation.
///
/// This is additional information that can be provided to robot code by the DS, such as colour information in 2020,
/// or switch/scale assignments in 2018.
///
/// This function will return -1 if either of the given pointers are null
/// It will return 1 if there was an error in the Rust code updating the GSM
/// It will return 0 on a success.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_game_specific_message(ds: *mut DriverStation, message: *const c_char) -> i8 {
    if ds.is_null() || message.is_null() {
        return -1;
    }

    let msg = CStr::from_ptr(message).to_str().unwrap();

    match (*ds).set_game_specific_message(msg) {
        Ok(()) => 0,
        Err(_) => 1
    }
}

/// Gets the robot mode of the specified ds, updating the value in `mode`
///
/// This function returns 1 if either pointer is NULL, and 0 on a success
/// On a success the value of `mode` will be updated with the current mode of the DS.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_get_mode(ds: *const DriverStation, mode: *mut Mode) -> u8 {
    if ds.is_null() || mode.is_null() {
        return 1;
    }

    *mode = Mode::from_ds((*ds).mode());
    return 0;
}

/// Changes the robot mode of the specified ds
///
/// If ds is NULL, this function does nothing.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_mode(ds: *mut DriverStation, mode: Mode) {
    if ds.is_null() {
        return;
    }

    (*ds).set_mode(mode.to_ds())
}

/// Gets the DsMode of the specified ds, DsMode can specify whether the DS is currently connected to a simulator
///
/// This function returns 1 if either pointer is NULL, and 0 on a success
/// On a successful function call, the value of `mode` will be updated with the current DsMode of the driver station.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_get_ds_mode(ds: *const DriverStation, mode: *mut DsMode) -> u8 {
    if ds.is_null() || mode.is_null() {
        return 1;
    }

    *mode = DsMode::from_ds((*ds).ds_mode());
    return 0;
}

/// Enables the robot connected to the given ds
///
/// This function does nothing if ds is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_enable(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    (*ds).enable();
}

/// Disables the robot connected to the given ds
///
/// This function does nothing if ds is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_disable(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    (*ds).disable();
}

/// Checks whether the given DS is enabling its connected robot
///
/// This function returns false if the pointer is NULL, and the true/false depending on whether the robot is enabled otherwise
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_enabled(ds: *const DriverStation) -> bool {
    if ds.is_null() {
        false
    } else {
        (*ds).enabled()
    }
}

/// Emergency stops the robot connected to the given ds
///
/// This function does nothing if ds is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_estop(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    (*ds).estop()
}

/// Checks whether the given ds is estopping its connected robot
///
/// This function returns false if ds is NULL, and the status reported by the driver station otherwise.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_estopped(ds: *const DriverStation) -> bool {
    if ds.is_null() {
        false
    } else {
        (*ds).estopped()
    }
}

/// Instructs the roboRIO connected to the given driver station to restart user code
///
/// This function does nothing if the given pointer is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_restart_code(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    (*ds).restart_code();
}

/// Instructs the roboRIO connected to the given driver station to reboot itself
///
/// This function does nothing if the given pointer is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_restart_roborio(ds: *mut DriverStation) {
    if ds.is_null() {
        return;
    }

    (*ds).restart_roborio();
}

/// Returns the latest Trace returned by the roboRIO connected to the given driver station
///
/// Trace is a bitflags value, the individual bitmasks are #define'd at the top of the header.
///
/// This function does nothing if the given pointer is NULL
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_trace(ds: *const DriverStation) -> u8 {
    if ds.is_null() {
        return 0;
    }

    return (*ds).trace().bits()
}

/// Returns the reported battery voltage of the connected robot
///
/// This function returns 0F if the given pointer is NULL, otherwise it returns the reported battery voltage
/// If no robot is connected this function will return 0F.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_battery_voltage(ds: *const DriverStation) -> f32 {
    if ds.is_null() {
        return 0f32;
    }

    return (*ds).battery_voltage()
}

/// Register a callback to be notified when the driver station returns TCP packets containing riolog data
///
/// This function does nothing if the given ds pointer is NULL
///
/// WARNING: The pointer passed to the callback is INVALIDATED after the callback returns
/// If keeping the string is desirable, it should be copied out of the pointer provided.
/// Keeping the raw pointer after the callback returns will result in a use-after-free bug when it
/// is next accessed.
#[no_mangle]
pub unsafe extern "C" fn DS_DriverStation_set_tcp_consumer(ds: *mut DriverStation, callback: extern "C" fn(StdoutMessage)) {
    if ds.is_null() {
        return;
    }

    (*ds).set_tcp_consumer(move |packet| {
        match packet {
            TcpPacket::Stdout(stdout) => {
                let cstr = CString::new(stdout.message).unwrap();
                let ffi = StdoutMessage::new(&cstr);
                callback(ffi);
            }
            _ => {}
        }
    })
}
