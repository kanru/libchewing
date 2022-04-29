//! Hsu keyboard layout

use super::KeyIndex;
use super::KeyIndex::*;

// FIXME: this thing should not be key indexes
pub struct KeyBuf(
    pub Option<KeyIndex>,
    pub Option<KeyIndex>,
    pub Option<KeyIndex>,
    pub Option<KeyIndex>,
);

pub fn is_hsu_end_key(key_buf: KeyBuf, key: KeyIndex) -> bool {
    match key {
        K28 | K29 | K30 | K33 | K48 => {
            key_buf.0.is_some() || key_buf.1.is_some() || key_buf.2.is_some()
        }
        _ => false,
    }
}
