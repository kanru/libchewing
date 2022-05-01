//! Standard keyboard layout
//!
//! Also known as the Dai Chien (大千) layout. It's the default layout on almost
//! all platforms and the most commonly used one.

use std::mem;

use crate::{
    bopomofo::{Bopomofo, BopomofoKind},
    keymap::{KeyEvent, KeyIndex},
};

use super::{KeyBehavior, KeyBuf, PhoneticKeyEditor};

pub struct Standard {
    key_buf: KeyBuf,
}

impl Standard {
    pub fn new() -> Standard {
        Standard {
            key_buf: KeyBuf(None, None, None, None),
        }
    }
    pub fn from_raw_parts(pho_inx: &[i32]) -> Standard {
        Standard {
            key_buf: KeyBuf(
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
            ),
        }
    }
}

impl PhoneticKeyEditor for Standard {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        let bopomofo = match key.index {
            KeyIndex::K1 => Bopomofo::B,
            KeyIndex::K2 => Bopomofo::D,
            KeyIndex::K3 => Bopomofo::TONE3,
            KeyIndex::K4 => Bopomofo::TONE4,
            KeyIndex::K5 => Bopomofo::ZH,
            KeyIndex::K6 => Bopomofo::TONE2,
            KeyIndex::K7 => Bopomofo::TONE5,
            KeyIndex::K8 => Bopomofo::A,
            KeyIndex::K9 => Bopomofo::AI,
            KeyIndex::K10 => Bopomofo::AN,
            KeyIndex::K11 => Bopomofo::ER,
            KeyIndex::K15 => Bopomofo::P,
            KeyIndex::K16 => Bopomofo::T,
            KeyIndex::K17 => Bopomofo::G,
            KeyIndex::K18 => Bopomofo::J,
            KeyIndex::K19 => Bopomofo::CH,
            KeyIndex::K20 => Bopomofo::Z,
            KeyIndex::K21 => Bopomofo::I,
            KeyIndex::K22 => Bopomofo::O,
            KeyIndex::K23 => Bopomofo::EI,
            KeyIndex::K24 => Bopomofo::EN,
            KeyIndex::K27 => Bopomofo::M,
            KeyIndex::K28 => Bopomofo::N,
            KeyIndex::K29 => Bopomofo::K,
            KeyIndex::K30 => Bopomofo::Q,
            KeyIndex::K31 => Bopomofo::SH,
            KeyIndex::K32 => Bopomofo::C,
            KeyIndex::K33 => Bopomofo::U,
            KeyIndex::K34 => Bopomofo::E,
            KeyIndex::K35 => Bopomofo::AU,
            KeyIndex::K36 => Bopomofo::ANG,
            KeyIndex::K38 => Bopomofo::F,
            KeyIndex::K39 => Bopomofo::L,
            KeyIndex::K40 => Bopomofo::H,
            KeyIndex::K41 => Bopomofo::X,
            KeyIndex::K42 => Bopomofo::R,
            KeyIndex::K43 => Bopomofo::S,
            KeyIndex::K44 => Bopomofo::IU,
            KeyIndex::K45 => Bopomofo::EH,
            KeyIndex::K46 => Bopomofo::OU,
            KeyIndex::K47 => Bopomofo::ENG,
            KeyIndex::K48 => Bopomofo::TONE1,
            _ => return KeyBehavior::KeyError,
        };
        let kind = bopomofo.kind();

        if kind == BopomofoKind::Tone {
            if self.key_buf.0.is_some()
                || self.key_buf.1.is_some()
                || self.key_buf.2.is_some()
                || self.key_buf.3.is_some()
            {
                return KeyBehavior::TryCommit;
            }
        } else {
            self.key_buf.3.take();
        }

        // In C libchewing TONE1 / Space is not a phonetic symbol
        if bopomofo == Bopomofo::TONE1 {
            return KeyBehavior::KeyError;
        }

        match kind {
            BopomofoKind::Initial => self.key_buf.0.replace(bopomofo),
            BopomofoKind::MedialGlide => self.key_buf.1.replace(bopomofo),
            BopomofoKind::Final => self.key_buf.2.replace(bopomofo),
            BopomofoKind::Tone => self.key_buf.3.replace(bopomofo),
        };
        KeyBehavior::Absorb
    }

    fn pop(&mut self) -> Option<Bopomofo> {
        if self.key_buf.3.is_some() {
            return mem::replace(&mut self.key_buf.3, None);
        } else if self.key_buf.2.is_some() {
            return mem::replace(&mut self.key_buf.2, None);
        } else if self.key_buf.1.is_some() {
            return mem::replace(&mut self.key_buf.1, None);
        } else if self.key_buf.0.is_some() {
            return mem::replace(&mut self.key_buf.0, None);
        }
        None
    }

    fn clear(&mut self) {
        self.key_buf = KeyBuf(None, None, None, None);
    }

    fn observe(&self) -> KeyBuf {
        self.key_buf
    }
}

#[cfg(test)]
mod test {
    use crate::{
        editor::phonetic::{KeyBehavior, PhoneticKeyEditor},
        keymap::{IdentityKeymap, KeyCode, Keymap, QWERTY},
    };

    use super::Standard;

    #[test]
    fn space() {
        let mut editor = Standard::new();
        let keymap = IdentityKeymap::new(QWERTY);
        let behavior = editor.key_press(keymap.map_key(KeyCode::Space));
        assert_eq!(KeyBehavior::KeyError, behavior);
    }
}
