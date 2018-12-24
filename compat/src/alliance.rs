use libc::{c_int, c_void};
use libds;

#[repr(C)]
pub struct Alliance {
    pub inner: *mut c_void
}

#[no_mangle]
pub extern "C" fn Alliance_new_red(pos: c_int) -> *mut Alliance {
    Box::into_raw(Box::new(Alliance {
        inner: Box::into_raw(Box::new(libds::Alliance::new_red(pos as u8))) as *mut c_void
    }))
}

#[no_mangle]
pub extern "C" fn Alliance_new_blue(pos: c_int) -> *mut Alliance {
    Box::into_raw(Box::new(Alliance {
        inner: Box::into_raw(Box::new(libds::Alliance::new_blue(pos as u8))) as *mut c_void
    }))
}
