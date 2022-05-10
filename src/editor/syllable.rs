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

use crate::{
    keymap::KeyEvent,
    zhuyin::{Bopomofo, Syllable},
};

pub use self::{
    dc26::DaiChien26,
    et26::Et26,
    hsu::Hsu,
    pinyin::{Pinyin, PinyinVariant},
    standard::Standard,
};

mod dc26;
mod et26;
mod hsu;
mod pinyin;
mod standard;

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

pub trait SyllableEditor: Debug {
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
    fn observe(&self) -> Syllable;
    /// Returns the current key seq buffer
    fn key_seq(&self) -> Option<String> {
        None
    }
}
