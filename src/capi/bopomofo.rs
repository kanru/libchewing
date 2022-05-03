use std::{
    ffi::{c_void, CString},
    os::raw::c_char,
    slice,
};

use crate::{
    editor::phonetic::{
        dc26::DaiChien26, et26::Et26, hsu::Hsu, pinyin::Pinyin, standard::Standard, KeyBehavior,
        KeyboardLayoutCompat, PhoneticKeyEditor,
    },
    keymap::{IdentityKeymap, KeyCode, KeyIndexFromQwerty, Keymap, QWERTY},
};

#[derive(Debug)]
#[repr(C)]
pub struct PhoneticKeyEditorWithKeymap {
    kb_type: KeyboardLayoutCompat,
    keymap: Box<dyn Keymap>,
    editor: Box<dyn PhoneticKeyEditor>,
}

#[no_mangle]
pub extern "C" fn NewPhoneticEditor(kb_type: KeyboardLayoutCompat) -> *mut c_void {
    use KeyboardLayoutCompat as KB;
    let editor: Box<PhoneticKeyEditorWithKeymap> = match kb_type {
        KB::Default => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Standard::new()),
        }),
        KB::Hsu => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Hsu::new()),
        }),
        KB::Ibm => todo!(),
        KB::GinYieh => todo!(),
        KB::Et => todo!(),
        KB::Et26 => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Et26::new()),
        }),
        KB::Dvorak => todo!(),
        KB::DvorakHsu => todo!(),
        KB::DachenCp26 => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(DaiChien26::new()),
        }),
        KB::HanyuPinyin => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::hanyu()),
        }),
        KB::ThlPinyin => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::thl()),
        }),
        KB::Mps2Pinyin => Box::new(PhoneticKeyEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::mps2()),
        }),
        KB::Carpalx => todo!(),
    };
    Box::into_raw(editor).cast()
}

#[no_mangle]
pub extern "C" fn FreePhoneticEditor(editor_keymap_ptr: *mut c_void) {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    unsafe { Box::from_raw(editor_keymap_ptr) };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorInput(editor_keymap_ptr: *mut c_void, key: i32) -> KeyBehavior {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let key_event = editor_keymap.keymap.map_key(key_code);
    let result = editor_keymap.editor.key_press(key_event);
    let key_buf = editor_keymap.editor.observe();

    if result == KeyBehavior::Commit {
        if key_buf.is_empty() {
            return if key_code == KeyCode::Space {
                KeyBehavior::KeyError
            } else {
                KeyBehavior::NoWord
            };
        }
        // FIXME make sure editors fills the tone
        // FIXME if dictionary doesn't have a word, return NO_WORD
    }

    result
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllable(editor_keymap_ptr: *mut c_void, pho_inx: *mut i32) {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    let syllable = editor_keymap.editor.observe();

    pho_inx[0] = match syllable.initial {
        Some(b) => b.initial_index(),
        None => 0,
    };
    pho_inx[1] = match syllable.medial {
        Some(b) => b.medial_index(),
        None => 0,
    };
    pho_inx[2] = match syllable.rime {
        Some(b) => b.rime_index(),
        None => 0,
    };
    pho_inx[3] = match syllable.tone {
        Some(b) => b.tone_index(),
        None => 0,
    };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableAlt(editor_keymap_ptr: *mut c_void, pho_inx: *mut i32) {
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    // FIXME
    let syllable = editor_keymap.editor.observe();

    pho_inx[0] = match syllable.initial {
        Some(b) => b.initial_index(),
        None => 0,
    };
    pho_inx[1] = match syllable.medial {
        Some(b) => b.medial_index(),
        None => 0,
    };
    pho_inx[2] = match syllable.rime {
        Some(b) => b.rime_index(),
        None => 0,
    };
    pho_inx[3] = match syllable.tone {
        Some(b) => b.tone_index(),
        None => 0,
    };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorKeyseq(editor_keymap_ptr: *mut c_void, key_seq: *mut c_char) {
    let key_seq = unsafe { slice::from_raw_parts_mut(key_seq as *mut u8, 10) };
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    if let Some(key_seq_str) = editor_keymap.editor.key_seq() {
        let key_seq_cstr = CString::new(key_seq_str).unwrap();
        let key_seq_bytes = key_seq_cstr.as_bytes_with_nul();
        key_seq[..key_seq_bytes.len()].copy_from_slice(key_seq_bytes);
    }
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableIndex(editor_keymap_ptr: *mut c_void) -> u16 {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    let syllable = editor_keymap.editor.observe();
    syllable.as_u16()
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableIndexAlt(editor_keymap_ptr: *mut c_void) -> u16 {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    // FIXME
    let syllable = editor_keymap.editor.observe();
    syllable.as_u16()
}

#[no_mangle]
pub extern "C" fn PhoneticEditorRemoveLast(editor_keymap_ptr: *mut c_void) {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    editor_keymap.editor.pop();
}

#[no_mangle]
pub extern "C" fn PhoneticEditorRemoveAll(editor_keymap_ptr: *mut c_void) {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    editor_keymap.editor.clear();
}

#[no_mangle]
pub extern "C" fn PhoneticEditorKbType(editor_keymap_ptr: *mut c_void) -> i32 {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    editor_keymap.kb_type as i32
}

#[no_mangle]
pub extern "C" fn PhoneticEditorIsEntering(editor_keymap_ptr: *mut c_void) -> bool {
    let editor_keymap_ptr: *mut PhoneticKeyEditorWithKeymap = editor_keymap_ptr.cast();
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut() }.unwrap();
    editor_keymap.editor.is_entering()
}
