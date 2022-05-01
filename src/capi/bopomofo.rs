use std::slice;

use crate::{
    editor::phonetic::{
        dc26::DaiChien26, et26::Et26, hsu::Hsu, standard::Standard, KeyBehavior, PhoneticKeyEditor,
    },
    keymap::{IdentityKeymap, KeyIndexFromQwerty, Keymap, QWERTY},
};

#[no_mangle]
pub extern "C" fn StandardInputRust(pho_inx: *mut i32, key: i32) -> KeyBehavior {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let mut editor = Standard::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = editor.key_press(event);
    let key_buf = editor.read();
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
pub extern "C" fn Et26PhoInputRust(pho_inx: *mut i32, key: i32) -> KeyBehavior {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let mut editor = Et26::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = editor.key_press(event);
    let key_buf = editor.read();
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
    let mut editor = Hsu::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = editor.key_press(event);
    let key_buf = editor.read();
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
pub extern "C" fn Dc26PhoInputRust(pho_inx: *mut i32, key: i32) -> KeyBehavior {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let mut editor = DaiChien26::from_raw_parts(pho_inx);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = editor.key_press(event);
    let key_buf = editor.read();
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
