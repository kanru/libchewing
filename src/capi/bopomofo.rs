use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    slice,
};

use crate::{
    editor::phonetic::{
        dc26::DaiChien26,
        et26::Et26,
        hsu::Hsu,
        pinyin::{Pinyin, PinyinVariant},
        standard::Standard,
        KeyBehavior, PhoneticKeyEditor,
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

#[no_mangle]
pub extern "C" fn PinYinInputRust(
    kb_type: i32,
    key_seq: *mut c_char,
    pho_inx: *mut i32,
    pho_inx_alt: *mut i32,
    key: i32,
) -> KeyBehavior {
    let key_seq_str = unsafe { CStr::from_ptr(key_seq) };
    let key_seq_slice = unsafe { slice::from_raw_parts_mut(key_seq as *mut u8, 10) };
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let pho_inx_alt = unsafe { slice::from_raw_parts_mut(pho_inx_alt, 4) };
    let kb_type = match kb_type {
        9 => PinyinVariant::HanyuPinyin,
        10 => PinyinVariant::ThlPinyin,
        11 => PinyinVariant::Mps2Pinyin,
        _ => panic!("non pinyin keyboard"),
    };
    let mut editor = Pinyin::from_raw_parts(kb_type, key_seq_str, pho_inx, pho_inx_alt);
    let keymap = IdentityKeymap::new(QWERTY);
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let event = keymap.map_key(key_code);
    let result = editor.key_press(event);
    if result == KeyBehavior::TryCommit {
        key_seq_slice[0] = 0;
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
        let alt = editor.alt();
        pho_inx_alt[0] = match alt.0 {
            Some(b) => b.initial_index(),
            None => 0,
        };
        pho_inx_alt[1] = match alt.1 {
            Some(b) => b.medial_index(),
            None => 0,
        };
        pho_inx_alt[2] = match alt.2 {
            Some(b) => b.final_index(),
            None => 0,
        };
        pho_inx_alt[3] = match alt.3 {
            Some(b) => b.tone_index(),
            None => 0,
        };
    } else {
        dbg!(editor.key_seq());
        let key_seq_cstr = CString::new(editor.key_seq().as_str()).unwrap();
        let key_seq_bytes = key_seq_cstr.as_bytes_with_nul();
        key_seq_slice[..key_seq_bytes.len()].copy_from_slice(key_seq_bytes);
    }
    result
}
