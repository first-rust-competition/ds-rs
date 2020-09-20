use ds::{JoystickValue, DriverStation};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref JOYSTICKS: Mutex<Vec<Vec<JoystickValue>>> = Mutex::new(Vec::with_capacity(6));
}

/// The error value returned by joystick functions if the specified port is out of bounds.
pub const EOUTOFBOUND: u8 = 1;
/// The error value returned by joystick functions if the joysticks Mutex was poisoned
pub const EPOISONLOCK: u8 = 2;

macro_rules! safe_unwrap_mux {
    () => {
        match JOYSTICKS.lock() {
            Ok(joy) => joy,
            Err(_) => return EPOISONLOCK,
        }
    }
}

/// Initializes the joystick supplier for the given DriverStation
/// After this is called, joystick values set with this API will be sent to any connected roboRIOs.
///
/// This function should only be called with a pointer returned from `DS_DriverStation_new_team` or `DS_DriverStation_new_ip`.
///
/// Returns:
/// -1 if the given pointer is NULL
/// 0 if the operation was a success.
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_init(ds: *mut DriverStation) -> i8 {
    if ds.is_null() {
        return -1;
    }

    (*ds).set_joystick_supplier(|| {
        JOYSTICKS.lock().unwrap().clone()
    });
    0
}

/// Attaches a new joystick, creating the new vector for it.
/// After calling this function, `port` can be used in the set_* functions to update values from the joystick
///
/// Returns:
/// `EOUTOFBOUND` if the specified port is greater than 5 (RIO only supports 6 joysticks)
/// `EPOISONLOCK` if the Mutex that stores the joysticks data was poisoned.
/// 0 if the operation was a success.
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_attach(port: usize) -> u8 {
    if port > 5 {
        return EOUTOFBOUND;
    }

    let mut joy = safe_unwrap_mux!();
    for _ in 0..port - joy.len() {
        joy.push(vec![])
    }
    0
}

/// Detaches a joystick, removing all its entries from the DS
/// After calling this function, `port` should **not** be used with set_* functions
/// If there are joysticks bound to ports greater than that specified, the vector may not be deleted,
/// however its contents will be cleared.
///
/// Returns:
/// `EOUTOFBOUND` if the specified port is greater than 5. (RIO only supports 6 joysticks).
/// `EPOISONLOCK` if the Mutex that stores the joystick data was poisoned.
/// 0 if the operation was a success
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_detach(port: usize) -> u8 {
    if port > 5 {
        return EOUTOFBOUND;
    }

    let mut joy = safe_unwrap_mux!();
    if port == joy.len() - 1 {
        joy.remove(port);
    } else {
        if let Some(js) = joy.get_mut(port) {
            *js = vec![];
        }
    }
    0
}

/// Updates the value of a button associated with the joystick on port `port`.
/// This function should only be used if `port` has been registered with `DS_Joystick_attach`
///
/// Returns:
/// `EOUTOFBOUND` if there is no vector stored at index `port`
/// `EPOISONLOCK` if the Mutex that stores joystick data was poisoned.
/// 0 if the operation was a success
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_set_button(port: usize, button: u8, pressed: bool) -> u8 {
    let mut joy = safe_unwrap_mux!();

    if port > joy.len() - 1 {
        return EOUTOFBOUND;
    }

    let mut inner = &mut joy[port];
    match inner.iter_mut().find(|value| value.is_button() && value.id() == button) {
        Some(btn) => *btn = JoystickValue::Button { id: button, pressed },
        None => inner.push(JoystickValue::Button { id: button, pressed }),
    }
    0
}

/// Updates the value of an axis associated with the joystick on port `port`
/// This function should only be used if `port` has been registered with `DS_Joystick_attach`
///
/// Returns:
/// `EOUTOFBOUND` if there is no vector stored at index `port`
/// `EPOISONLOCK` if the Mutex that stores joystick data was poisoned
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_set_axis(port: usize, axis: u8, value: f32) -> u8 {
    let mut joy = safe_unwrap_mux!();

    if port > joy.len() - 1 {
        return EOUTOFBOUND;
    }

    let mut inner = &mut joy[port];
    match inner.iter_mut().find(|value| value.is_axis() && value.id() == axis) {
        Some(ax) => *ax = JoystickValue::Axis { id: axis, value },
        None => inner.push(JoystickValue::Axis { id: axis, value }),
    }
    0
}

/// Updates the value of a POV, or d-pad associated with the joystick on port `port`
/// This function should only be used if `port` has been registered with `DS_Joystick_attach`
///
/// Returns:
/// `EOUTOFBOUND` if there is no vector stored at index `port`
/// `EPOISONLOCK` if the Mutex that stores joystick data was poisoned
#[no_mangle]
pub unsafe extern "C" fn DS_Joystick_set_pov(port: usize, pov: u8, value: i16) -> u8 {
    let mut joy = safe_unwrap_mux!();

    if port > joy.len() - 1 {
        return EOUTOFBOUND;
    }

    let mut inner = &mut joy[port];
    match inner.iter_mut().find(|value| value.is_pov() && value.id() == pov) {
        Some(p) => *p = JoystickValue::POV { id: pov, angle: value },
        None => inner.push(JoystickValue::POV { id: pov, angle: value })
    }
    0
}