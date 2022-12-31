use std::ffi::{c_char, c_int};

use ffi_opaque::opaque;

opaque! {
    pub(crate) struct ChewingData;
}

extern "C" {
    pub(crate) fn FillPreeditBuf(pgdata: *mut ChewingData, phrase: *mut c_char, from: c_int, to: c_int);
}
