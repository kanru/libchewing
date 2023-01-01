use std::ffi::{c_int};

use super::types::ChewingData;

extern "C" {
    pub fn toPreeditBufIndex(pgdata: *mut ChewingData, pos: c_int) -> c_int;
    pub fn HaninSymbolInput(pgdata: *mut ChewingData) -> c_int;
}
