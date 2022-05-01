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

use std::ops::Shl;

use crate::{bopomofo::Bopomofo, keymap::KeyEvent};

pub mod et26;
pub mod hsu;
pub mod standard;

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
    TryCommit,
}

#[derive(Clone, Copy, Default)]
pub struct KeyBuf(
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
    pub Option<Bopomofo>,
);

impl KeyBuf {
    pub fn encode(&self) -> u16 {
        (self.0.unwrap_or(Bopomofo::B) as u16).shl(9)
            + (self.1.unwrap_or(Bopomofo::B) as u16).shl(7)
            + (self.2.unwrap_or(Bopomofo::B) as u16).shl(3)
            + (self.3.unwrap_or(Bopomofo::B) as u16)
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

pub trait PhoneticKeyEditor {
    /// Handles a key press event and returns the behavior of the layout.
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior;
    /// Removes the last phonetic key from the buffer and returns it, or [`None`] if it
    /// is empty.
    fn pop(&mut self) -> Option<Bopomofo>;
    /// Clears the phonetic key buffer, removing all values.
    fn clear(&mut self);
    /// Returns the current phonetic key buffer without changing it.
    fn observe(&self) -> KeyBuf;
    /// Returns the current phonetic key buffer and clears it.
    fn read(&mut self) -> KeyBuf {
        let keybuf = self.observe();
        self.clear();
        keybuf
    }
}
