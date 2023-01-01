use std::ffi::c_int;

pub const CHINESE_MODE: usize = 1;
pub const SYMBOL_MODE: usize = 0;
pub const FULLSHAPE_MODE: usize = 1;
pub const HALFSHAPE_MODE: usize = 0;
pub const AUTOLEARN_DISABLED: usize = 1;
pub const AUTOLEARN_ENABLED: usize = 0;

pub const MIN_SELKEY: usize = 1;
pub const MAX_SELKEY: usize = 10;

pub const CHEWING_LOG_VERBOSE: usize = 1;
pub const CHEWING_LOG_DEBUG: usize = 2;
pub const CHEWING_LOG_INFO: usize = 3;
pub const CHEWING_LOG_WARN: usize = 4;
pub const CHEWING_LOG_ERROR: usize = 5;

/// Use "asdfjkl789" as selection key
pub const HSU_SELKEY_TYPE1: usize = 1;
/// Use "asdfzxcv89" as selection key
pub const HSU_SELKEY_TYPE2: usize = 2;

/// cbindgen:prefix-with-name
/// cbindgen:enum-trailing-values=[Count]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum KB {
    Default,
    Hsu,
    Ibm,
    GinYieh,
    Et,
    Et26,
    Dvorak,
    DvorakHsu,
    DachenCp26,
    HanyuPinyin,
    ThlPinyin,
    Mps2Pinyin, /* Mandarin Phonetic Symbols II */
    Carpalx,
}

/// Deprecated, use chewing_set_ series of functions to set parameters instead.
/// cbindgen:rename-all=CamelCase
#[repr(C)]
pub struct ChewingConfigData {
    pub cand_per_page: c_int,
    pub max_chi_symbol_len: c_int,
    pub sel_key: [c_int; MAX_SELKEY],
    pub b_add_phrase_forward: c_int,
    pub b_space_as_selection: c_int,
    pub b_esc_clean_all_buf: c_int,
    pub b_auto_shift_cur: c_int,
    pub b_easy_symbol_input: c_int,
    pub b_phrase_choice_rearward: c_int,
    pub b_auto_learn: c_int,
    pub hsu_sel_key_type: c_int,
}

#[repr(C)]
pub struct IntervalType {
    pub from: c_int,
    pub to: c_int,
}