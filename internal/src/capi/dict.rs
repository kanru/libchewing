use std::{
    ffi::{c_void, CStr, CString},
    fs::File,
    os::raw::{c_char, c_int},
    path::Path,
    slice,
};

use chewing::dictionary::{ChainedDictionary, Dictionary, TrieDictionary};

#[no_mangle]
pub extern "C" fn InitDict(prefix: *mut c_char) -> *mut c_void {
    let prefix = unsafe { CStr::from_ptr(prefix) }
        .to_str()
        .expect("Invalid prefix string");
    let path: &Path = prefix.as_ref();

    let mut tsi_db_path = path.to_path_buf();
    tsi_db_path.push("tsi.dat");
    let tsi_db_file = File::open(tsi_db_path).expect("Unable to open file");
    let tsi_db = Box::new(TrieDictionary::new(tsi_db_file).expect("Unable to parse tsi db"));

    let mut word_db_path = path.to_path_buf();
    word_db_path.push("word.dat");
    let word_db_file = File::open(word_db_path).expect("Unable to open file");
    let word_db = Box::new(TrieDictionary::new(word_db_file).expect("Unable to parse word db"));

    let dict = Box::new(ChainedDictionary::new(vec![word_db, tsi_db], vec![]));

    Box::into_raw(dict).cast()
}

#[no_mangle]
pub extern "C" fn TerminateDict(dict_ptr: *mut c_void) {
    let dict_ptr: *mut ChainedDictionary = dict_ptr.cast();
    unsafe { Box::from_raw(dict_ptr) };
}

#[repr(C)]
pub struct Phrase {
    phrase: [u8; 11 * 4 + 1],
    freq: c_int,
}

#[no_mangle]
pub extern "C" fn GetCharFirst(
    dict_ptr: *mut c_void,
    phrase_ptr: *mut Phrase,
    syllable_u16: u16,
) -> *mut c_void {
    let dict_ptr: *mut ChainedDictionary = dict_ptr.cast();
    let dict = unsafe { dict_ptr.as_ref() }.expect("Null pointer");
    let syllable = syllable_u16
        .try_into()
        .expect("Unable to convert u16 to syllable");
    let words = dict.lookup_word(syllable).collect::<Vec<_>>();
    let mut iter = Box::new(
        Box::new(words.into_iter()) as Box<dyn Iterator<Item = chewing::dictionary::Phrase>>
    );
    if let Some(phrase) = iter.next() {
        let c_phrase = unsafe { &mut *phrase_ptr };
        let phrase_str = CString::new(phrase.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        c_phrase.freq = phrase.freq() as c_int;
        c_phrase.phrase[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
        return Box::into_raw(iter).cast();
    }

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn GetPhraseFirst(vec_ptr: *mut c_void, phrase_ptr: *mut Phrase) -> *mut c_void {
    let vec_ptr: *mut Vec<chewing::dictionary::Phrase> = vec_ptr.cast();
    let vec = unsafe { Box::from_raw(vec_ptr) };
    let new_vec = vec.clone();
    // FIXME Leak the vec because it's reused
    Box::into_raw(vec);
    let mut iter = Box::new(
        Box::new(new_vec.into_iter()) as Box<dyn Iterator<Item = chewing::dictionary::Phrase>>
    );
    if let Some(phrase) = iter.next() {
        let c_phrase = unsafe { &mut *phrase_ptr };
        let phrase_str = CString::new(phrase.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        c_phrase.freq = phrase.freq() as c_int;
        c_phrase.phrase[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
        return Box::into_raw(iter).cast();
    }

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn TreeFindPhrase(
    dict_ptr: *mut c_void,
    begin: c_int,
    end: c_int,
    syllables_u16: *mut u16,
) -> *mut c_void {
    let dict_ptr: *mut ChainedDictionary = dict_ptr.cast();
    let dict = unsafe { dict_ptr.as_ref() }.expect("Null pointer");
    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16, 50) };
    let begin = begin as usize;
    let end = end as usize;
    let syllables = syllables_u16[begin..=end]
        .iter()
        .map(|&syl_u16| {
            syl_u16
                .try_into()
                .expect("Unable to convert u16 to syllable")
        })
        .collect::<Vec<_>>();
    let phrases = Box::new(dict.lookup_phrase(syllables.as_slice()).collect::<Vec<_>>());
    if !phrases.is_empty() {
        return Box::into_raw(phrases).cast();
    }

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn GetVocabNext(iter_ptr: *mut c_void, phrase_ptr: *mut Phrase) -> *mut c_void {
    let iter_ptr: *mut Box<dyn Iterator<Item = chewing::dictionary::Phrase>> = iter_ptr.cast();
    let mut iter = unsafe { Box::from_raw(iter_ptr) };
    if let Some(phrase) = iter.next() {
        let c_phrase = unsafe { &mut *phrase_ptr };
        let phrase_str = CString::new(phrase.as_str()).expect("Unable to convert to CString");
        let phrase_bytes = phrase_str.as_bytes_with_nul();
        c_phrase.freq = phrase.freq() as c_int;
        c_phrase.phrase[0..phrase_bytes.len()].copy_from_slice(phrase_bytes);
        return Box::into_raw(iter).cast();
    }
    std::ptr::null_mut()
}
