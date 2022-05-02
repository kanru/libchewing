//! Conversion from key events to phonetic keys
//!
//! This module contains engines for phonetic key conversions.
//!
//! Traditionally the keyboards sold in Chinese speaking region have
//! both the English alphabets and Zhuyin symbols printed on the keys.
//! Like English keyboards can have different layouts (QWERTY, Dvorak, etc.),
//! Zhuyin keyboards also have different layouts.
//!
//! The most widely used Zhuyin layout is the one directly printed on the keyboards.
//! It is a one to one mapping from keys to Zhuyin symbols. However, some layouts
//! have smarter mapping from keys to Zhuyin symbols, taking advantage of impossible
//! combinations, to reduce the total keys required.
//!
//! Chewing currently supports the default layout, Hsu's layout, ET26 layout,
//! DaChen CP26 layout, and the Pinyin layout.

use std::fmt::Debug;

use crate::{bopomofo::Bopomofo, keymap::KeyEvent};

pub mod dc26;
pub mod et26;
pub mod hsu;
pub mod pinyin;
pub mod standard;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum KeyboardLayoutCompat {
    Default = 0,
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
    Mps2Pinyin,
    Carpalx,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum KeyBehavior {
    Ignore = 0,
    Absorb,
    Commit,
    KeyError,
    Error,
    NoWord,
    OpenSymbolTable,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct KeyBuf(
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
);

impl KeyBuf {
    pub fn is_empty(&self) -> bool {
        self.0.is_none() && self.1.is_none() && self.2.is_none() && self.3.is_none()
    }
    pub fn encode(&self) -> u16 {
        let initial = self.0.map_or(0, |v| v as u16 + 1);
        let medial = self.1.map_or(0, |v| (v as u16) - 20);
        let r#final = self.2.map_or(0, |v| (v as u16) - 23);
        let tone = self.3.map_or(0, |v| (v as u16) - 36).clamp(0, 4);
        (initial << 9) | (medial << 7) | (r#final << 3) | tone
    }
    pub fn from_raw_parts(pho_inx: &[i32]) -> KeyBuf {
        KeyBuf(
            if pho_inx[0] == 0 {
                None
            } else {
                Some(Bopomofo::from_initial(pho_inx[0]))
            },
            if pho_inx[1] == 0 {
                None
            } else {
                Some(Bopomofo::from_medial(pho_inx[1]))
            },
            if pho_inx[2] == 0 {
                None
            } else {
                Some(Bopomofo::from_final(pho_inx[2]))
            },
            if pho_inx[3] == 0 {
                None
            } else {
                Some(Bopomofo::from_tone(pho_inx[3]))
            },
        )
    }
}

pub trait PhoneticKeyEditor: Debug {
    /// Handles a key press event and returns the behavior of the layout.
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior;
    /// Returns whether the editor contains any input.
    fn is_entering(&self) -> bool;
    /// Removes the last phonetic key from the buffer and returns it, or [`None`] if it
    /// is empty.
    fn pop(&mut self) -> Option<Bopomofo>;
    /// Clears the phonetic key buffer, removing all values.
    fn clear(&mut self);
    /// Returns the current phonetic key buffer without changing it.
    fn observe(&self) -> KeyBuf;
    /// Returns the current key seq buffer
    fn key_seq(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::bopomofo::Bopomofo;

    use super::KeyBuf;

    #[test]
    fn encode_hsu_sdf() {
        let key_buf = KeyBuf(Some(Bopomofo::S), None, None, None);
        let syllable_code = key_buf.encode();
        assert_eq!(0x2A00, syllable_code);

        let key_buf = KeyBuf(Some(Bopomofo::D), None, None, None);
        let syllable_code = key_buf.encode();
        assert_eq!(0xA00, syllable_code);

        let key_buf = KeyBuf(Some(Bopomofo::F), None, None, None);
        let syllable_code = key_buf.encode();
        assert_eq!(0x800, syllable_code);
    }
}
