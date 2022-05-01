use std::slice;

use crate::{
    editor::phonetic::{hsu::Hsu, standard::Standard, KeyBehavior, PhoneticKeyEditor},
    keymap::{IdentityKeymap, KeyIndexFromQwerty, Keymap, QWERTY},
};

#[no_mangle]
pub extern "C" fn StandardInputRust(pho_inx: *mut i32, key: i32) -> KeyBehavior {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let mut standard = Standard::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = standard.key_press(event);
    let key_buf = standard.read();
    pho_inx[0] = match key_buf.0 {
        Some(b) => b.initial_index(),
        None => 0,
    };
    pho_inx[1] = match key_buf.1 {
        Some(b) => b.medial_index(),
        None => 0,
    };
    pho_inx[2] = match key_buf.2 {
        Some(b) => b.final_index(),
        None => 0,
    };
    pho_inx[3] = match key_buf.3 {
        Some(b) => b.tone_index(),
        None => 0,
    };
    result
}

#[no_mangle]
pub extern "C" fn HsuPhoInputRust(pho_inx: *mut i32, key: i32) -> KeyBehavior {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let mut hsu = Hsu::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = hsu.key_press(event);
    let key_buf = hsu.read();
    pho_inx[0] = match key_buf.0 {
        Some(b) => b.initial_index(),
        None => 0,
    };
    pho_inx[1] = match key_buf.1 {
        Some(b) => b.medial_index(),
        None => 0,
    };
    pho_inx[2] = match key_buf.2 {
        Some(b) => b.final_index(),
        None => 0,
    };
    pho_inx[3] = match key_buf.3 {
        Some(b) => b.tone_index(),
        None => 0,
    };
    result
}
