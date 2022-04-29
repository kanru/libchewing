use std::slice;

use crate::keymap::{
    hsu::{is_hsu_end_key, KeyBuf},
    KeyIndexFromQwerty,
};

#[no_mangle]
pub extern "C" fn IsHsuPhoEndKey(pho_inx: *const i32, key: i32) -> bool {
    let pho_inx = unsafe { slice::from_raw_parts(pho_inx, 4) };
    let key_buf = KeyBuf(
        if pho_inx[0] != 0 {
            Some((pho_inx[0] as u8).as_key_index())
        } else {
            None
        },
        if pho_inx[1] != 0 {
            Some((pho_inx[1] as u8).as_key_index())
        } else {
            None
        },
        if pho_inx[2] != 0 {
            Some((pho_inx[2] as u8).as_key_index())
        } else {
            None
        },
        if pho_inx[3] != 0 {
            Some((pho_inx[3] as u8).as_key_index())
        } else {
            None
        },
    );
    is_hsu_end_key(key_buf, (key as u8).as_key_index())
}
