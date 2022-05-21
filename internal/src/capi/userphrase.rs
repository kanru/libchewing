use std::{
    ffi::{c_void, CStr, CString},
    iter::Peekable,
    os::raw::{c_char, c_int, c_uint},
    path::Path,
    slice,
};

use chewing::{
    dictionary::{DictEntries, Dictionary, SqliteDictionary},
    editor::{SqliteUserFreqEstimate, UserFreqEstimate},
    zhuyin::Syllable,
};

struct UserphraseDbAndEstimate {
    db: SqliteDictionary,
    estimate: SqliteUserFreqEstimate,
}

#[no_mangle]
pub extern "C" fn InitUserphrase(path: *mut c_char) -> *mut c_void {
    let path = unsafe { CStr::from_ptr(path) }
        .to_str()
        .expect("Invalid prefix string");
    let path: &Path = path.as_ref();

    let chewing_db = match SqliteDictionary::open(&path) {
        Ok(db) => db,
        Err(_) => return std::ptr::null_mut(),
    };

    let estimate = match SqliteUserFreqEstimate::open(&path) {
        Ok(db) => db,
        Err(_) => return std::ptr::null_mut(),
    };

    Box::into_raw(Box::new(UserphraseDbAndEstimate {
        db: chewing_db,
        estimate,
    }))
    .cast()
}

#[no_mangle]
pub extern "C" fn TerminateUserphrase(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let ptr: *mut UserphraseDbAndEstimate = ptr.cast();
    unsafe { Box::from_raw(ptr) };
}

#[repr(C)]
pub struct Phrase {
    phrase: [u8; 11 * 4 + 1],
    freq: c_int,
}

#[repr(C)]
pub struct UserPhraseData {
    syllables_u16: [u16; 50],
    phrase: [u8; 11 * 4 + 1],
    user_freq: c_int,
    recent_time: c_int,
    orig_freq: c_int,
    max_freq: c_int,
}

#[no_mangle]
pub extern "C" fn UserGetPhraseFirst(
    ue_ptr: *mut c_void,
    userphrase_data_ptr: *mut c_void,
    syllables_u16_ptr: *mut u16,
) -> *mut c_void {
    let ue_ptr: *mut UserphraseDbAndEstimate = ue_ptr.cast();
    let ue = unsafe { ue_ptr.as_ref() }.expect("Null ptr");
    let userphrase_data_ptr: *mut UserPhraseData = userphrase_data_ptr.cast();
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let mut iter = Box::new(Box::new(
        ue.db
            .lookup_phrase(syllables.as_slice())
            .collect::<Vec<_>>()
            .into_iter(),
    )
        as Box<dyn Iterator<Item = chewing::dictionary::Phrase>>);

    if let Some(phrase) = iter.next() {
        let userphrase_data = unsafe { &mut *userphrase_data_ptr };
        let phrase_str = CString::new(phrase.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        // userphrase_data.syllables_u16 = syllables_u16_ptr;
        userphrase_data.user_freq = phrase.freq() as c_int;
        userphrase_data.phrase[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
        return Box::into_raw(iter).cast();
    }

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn UserGetPhraseNext(
    iter_ptr: *mut c_void,
    userphrase_data_ptr: *mut c_void,
) -> *mut c_void {
    let userphrase_data_ptr: *mut UserPhraseData = userphrase_data_ptr.cast();
    let iter_ptr: *mut Box<dyn Iterator<Item = chewing::dictionary::Phrase>> = iter_ptr.cast();
    let mut iter = unsafe { Box::from_raw(iter_ptr) };
    if let Some(phrase) = iter.next() {
        let userphrase_data = unsafe { &mut *userphrase_data_ptr };
        let phrase_str = CString::new(phrase.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        userphrase_data.user_freq = phrase.freq() as c_int;
        userphrase_data.phrase[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
        return Box::into_raw(iter).cast();
    }
    std::ptr::null_mut()
}

#[repr(u8)]
enum UserUpdateState {
    Fail = 4,
    Insert = 1,
    Modify = 2,
    Ignore = 8,
}

const C_API_MAX_USER_PHRASE_LEN: usize = 11;

#[no_mangle]
pub extern "C" fn UserUpdatePhrase(
    ue_ptr: *mut c_void,
    syllables_u16_ptr: *mut u16,
    phrase_str_ptr: *mut c_char,
) -> u8 {
    let ue_ptr: *mut UserphraseDbAndEstimate = ue_ptr.cast();
    let ue = unsafe { ue_ptr.as_mut() }.expect("Null ptr");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let phrase_str = unsafe { CStr::from_ptr(phrase_str_ptr) }
        .to_str()
        .expect("Invalid UTF-8 str");
    if syllables.len() > C_API_MAX_USER_PHRASE_LEN {
        return UserUpdateState::Fail as u8;
    }
    let phrases = ue.db.lookup_phrase(&syllables).collect::<Vec<_>>();
    if phrases.is_empty() {
        // FIXME provide max_freq, orig_freq
        ue.db
            .as_mut_dict()
            .unwrap()
            .insert(&syllables, (phrase_str, 1).into())
            .expect("SQL error");
        return UserUpdateState::Insert as u8;
    }
    let phrase = phrases
        .iter()
        .cloned()
        .find(|p| p.as_str() == phrase_str)
        .unwrap_or_else(|| (phrase_str, 1).into());
    let max_freq = phrases.iter().map(|p| p.freq()).max().unwrap();
    let user_freq = ue.estimate.estimate(&phrase, phrase.freq(), max_freq);
    let time = ue.estimate.now().unwrap();
    ue.db
        .as_mut_dict()
        .unwrap()
        .update(&syllables, phrase.clone(), user_freq, time)
        .expect("SQL error");
    UserUpdateState::Modify as u8
}

#[no_mangle]
pub extern "C" fn UserRemovePhrase(
    ue_ptr: *mut c_void,
    syllables_u16_ptr: *mut u16,
    phrase_str_ptr: *mut c_char,
) -> bool {
    let ue_ptr: *mut UserphraseDbAndEstimate = ue_ptr.cast();
    let ue = unsafe { ue_ptr.as_mut() }.expect("Null ptr");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, 50) };
    let syllables = syllables_u16
        .iter()
        .take_while(|&&syl_u16| syl_u16 != 0)
        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
        .collect::<Vec<_>>();
    let phrase_str = unsafe { CStr::from_ptr(phrase_str_ptr) }
        .to_str()
        .expect("Invalid UTF-8 str");
    let has_phrase = ue
        .db
        .lookup_phrase(&syllables)
        .any(|p| p.as_str() == phrase_str);
    ue.db
        .as_mut_dict()
        .unwrap()
        .remove(&syllables, phrase_str)
        .expect("SQL error");
    has_phrase
}

#[no_mangle]
pub extern "C" fn IncreaseLifeTime(ue_ptr: *mut c_void) {
    let ue_ptr: *mut UserphraseDbAndEstimate = ue_ptr.cast();
    let ue = unsafe { ue_ptr.as_mut() }.expect("Null ptr");
    ue.estimate.tick().expect("SQL error");
}

#[no_mangle]
pub extern "C" fn UserUpdatePhraseBegin(_: *mut c_void) {}
#[no_mangle]
pub extern "C" fn UserGetPhraseEnd(_: *mut c_void, _: *mut c_void) {}

#[no_mangle]
pub extern "C" fn UserEnumeratePhrase(ue_ptr: *mut c_void) -> *mut c_void {
    let ue_ptr: *mut UserphraseDbAndEstimate = ue_ptr.cast();
    let ue = unsafe { ue_ptr.as_mut() }.expect("Null ptr");
    Box::into_raw(Box::new(ue.db.entries().peekable()) as Box<Peekable<DictEntries>>).cast()
}

#[no_mangle]
pub extern "C" fn UserEnumerateHasNext(
    iter_ptr: *mut c_void,
    phrase_len_ptr: *mut c_uint,
    bopomofo_len: *mut c_uint,
) -> bool {
    let iter_ptr: *mut Peekable<DictEntries> = iter_ptr.cast();
    let iter = unsafe { iter_ptr.as_mut() }.expect("Null ptr");
    match iter.peek() {
        Some(entry) => {
            unsafe {
                phrase_len_ptr.write((entry.1.as_str().len() + 1) as u32);
                bopomofo_len.write(
                    (entry
                        .0
                        .iter()
                        .map(|syl| syl.to_string().len() + 1)
                        .sum::<usize>()
                        + 1) as u32,
                );
            }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn UserEnumerateGet(
    iter_ptr: *mut c_void,
    phrase_buf: *mut c_char,
    _phrase_len_ptr: *const c_uint,
    bopomofo_buf: *mut c_char,
    _bopomofo_len: *const c_uint,
) -> c_int {
    let iter_ptr: *mut Peekable<DictEntries> = iter_ptr.cast();
    let iter = unsafe { iter_ptr.as_mut() }.expect("Null ptr");
    match iter.next() {
        Some(entry) => {
            unsafe {
                let phrase_str = CString::new(entry.1.as_str()).unwrap();
                let phrase_str_bytes = phrase_str.as_bytes_with_nul();
                phrase_buf.copy_from(
                    phrase_str_bytes.as_ptr() as *const i8,
                    phrase_str_bytes.len(),
                );
                // phrase_len_ptr.write((entry.1.as_str().len() + 1) as u32);
                let bopomofo_str = CString::new(
                    entry
                        .0
                        .iter()
                        .map(|syl| syl.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
                .unwrap();
                let bopomofo_str_bytes = bopomofo_str.as_bytes_with_nul();
                bopomofo_buf.copy_from(
                    bopomofo_str_bytes.as_ptr() as *const i8,
                    bopomofo_str_bytes.len(),
                );
                // bopomofo_len.write(bopomofo_str_bytes.len() as u32);
            }
            0
        }
        None => 1,
    }
}
