use std::{ffi::CString, os::raw::c_char, slice};

use chewing::editor::{
    keymap::{IdentityKeymap, KeyCode, KeyCodeFromQwerty, Keymap, QWERTY, RemappingKeymap, DVORAK, CARPALX},
    layout::{DaiChien26, Et26, Hsu, Ibm, KeyBehavior, KeyboardLayoutCompat, Pinyin, Standard, Et, GinYieh},
    SyllableEditor,
};

#[repr(C)]
pub struct SyllableEditorWithKeymap {
    kb_type: KeyboardLayoutCompat,
    keymap: Box<dyn Keymap>,
    editor: Box<dyn SyllableEditor>,
}

#[no_mangle]
pub extern "C" fn NewPhoneticEditor(
    kb_type: KeyboardLayoutCompat,
) -> *mut SyllableEditorWithKeymap {
    use KeyboardLayoutCompat as KB;
    let editor: Box<SyllableEditorWithKeymap> = match kb_type {
        KB::Default => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Standard::new()),
        }),
        KB::Hsu => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Hsu::new()),
        }),
        KB::Ibm => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Ibm::new()),
        }),
        KB::GinYieh => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(GinYieh::new()),
        }),
        KB::Et => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Et::new()),
        }),
        KB::Et26 => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Et26::new()),
        }),
        KB::Dvorak => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(DVORAK, QWERTY)),
            editor: Box::new(Standard::new()),
        }),
        KB::DvorakHsu => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(DVORAK, QWERTY)),
            editor: Box::new(Hsu::new()),
        }),
        KB::DachenCp26 => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(DaiChien26::new()),
        }),
        KB::HanyuPinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::hanyu()),
        }),
        KB::ThlPinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::thl()),
        }),
        KB::Mps2Pinyin => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(IdentityKeymap::new(QWERTY)),
            editor: Box::new(Pinyin::mps2()),
        }),
        KB::Carpalx => Box::new(SyllableEditorWithKeymap {
            kb_type,
            keymap: Box::new(RemappingKeymap::new(CARPALX, QWERTY)),
            editor: Box::new(Standard::new()),
        }),
    };
    Box::into_raw(editor)
}

#[no_mangle]
pub extern "C" fn FreePhoneticEditor(editor_keymap_ptr: *mut SyllableEditorWithKeymap) {
    unsafe { Box::from_raw(editor_keymap_ptr) };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorInput(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
    key: i32,
) -> KeyBehavior {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    let key_code = match (key as u8).as_key_code() {
        Some(key_code) => key_code,
        None => return KeyBehavior::KeyError,
    };
    let key_event = editor_keymap.keymap.map_key(key_code);
    let result = editor_keymap.editor.key_press(key_event);
    let key_buf = editor_keymap.editor.read();

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
pub extern "C" fn PhoneticEditorSyllable(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
    pho_inx: *mut i32,
) {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    let syllable = editor_keymap.editor.read();

    pho_inx[0] = match syllable.initial {
        Some(b) => b.initial_index() as i32,
        None => 0,
    };
    pho_inx[1] = match syllable.medial {
        Some(b) => b.medial_index() as i32,
        None => 0,
    };
    pho_inx[2] = match syllable.rime {
        Some(b) => b.rime_index() as i32,
        None => 0,
    };
    pho_inx[3] = match syllable.tone {
        Some(b) => b.tone_index() as i32,
        None => 0,
    };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableAlt(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
    pho_inx: *mut i32,
) {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    let pho_inx = unsafe { slice::from_raw_parts_mut(pho_inx, 4) };
    // FIXME
    let syllable = editor_keymap.editor.read();

    pho_inx[0] = match syllable.initial {
        Some(b) => b.initial_index() as i32,
        None => 0,
    };
    pho_inx[1] = match syllable.medial {
        Some(b) => b.medial_index() as i32,
        None => 0,
    };
    pho_inx[2] = match syllable.rime {
        Some(b) => b.rime_index() as i32,
        None => 0,
    };
    pho_inx[3] = match syllable.tone {
        Some(b) => b.tone_index() as i32,
        None => 0,
    };
}

#[no_mangle]
pub extern "C" fn PhoneticEditorKeyseq(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
    key_seq: *mut c_char,
) {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    let key_seq = unsafe { slice::from_raw_parts_mut(key_seq as *mut u8, 10) };
    if let Some(key_seq_str) = editor_keymap.editor.key_seq() {
        let key_seq_cstr = CString::new(key_seq_str).unwrap();
        let key_seq_bytes = key_seq_cstr.as_bytes_with_nul();
        key_seq[..key_seq_bytes.len()].copy_from_slice(key_seq_bytes);
    }
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableIndex(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
) -> u16 {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    let syllable = editor_keymap.editor.read();
    syllable.to_u16()
}

#[no_mangle]
pub extern "C" fn PhoneticEditorSyllableIndexAlt(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
) -> u16 {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    // FIXME
    let syllable = editor_keymap.editor.read();
    syllable.to_u16()
}

#[no_mangle]
pub extern "C" fn PhoneticEditorRemoveLast(editor_keymap_ptr: *mut SyllableEditorWithKeymap) {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    editor_keymap.editor.remove_last();
}

#[no_mangle]
pub extern "C" fn PhoneticEditorRemoveAll(editor_keymap_ptr: *mut SyllableEditorWithKeymap) {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    editor_keymap.editor.clear();
}

#[no_mangle]
pub extern "C" fn PhoneticEditorKbType(editor_keymap_ptr: *mut SyllableEditorWithKeymap) -> i32 {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    editor_keymap.kb_type as i32
}

#[no_mangle]
pub extern "C" fn PhoneticEditorIsEntering(
    editor_keymap_ptr: *mut SyllableEditorWithKeymap,
) -> bool {
    let editor_keymap = unsafe { editor_keymap_ptr.as_mut().unwrap() };
    !editor_keymap.editor.is_empty()
}
