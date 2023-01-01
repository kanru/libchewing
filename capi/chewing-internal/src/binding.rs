use std::ffi::{c_char, c_int};

use super::types::ChewingData;

extern "C" {
    pub fn FillPreeditBuf(pgdata: *mut ChewingData, phrase: *mut c_char, from: c_int, to: c_int);
}
