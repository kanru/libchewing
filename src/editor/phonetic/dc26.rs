//! Dai Chien CP26

use crate::{
    bopomofo::{Bopomofo, BopomofoKind},
    keymap::{KeyEvent, KeyIndex},
};

use super::{KeyBehavior, KeyBuf, PhoneticKeyEditor};

#[derive(Debug)]
pub struct DaiChien26 {
    key_buf: KeyBuf,
}

impl DaiChien26 {
    pub fn new() -> DaiChien26 {
        DaiChien26 {
            key_buf: Default::default(),
        }
    }
    pub fn from_raw_parts(pho_inx: &[i32]) -> DaiChien26 {
        DaiChien26 {
            key_buf: KeyBuf::from_raw_parts(pho_inx),
        }
    }
    fn is_end_key(&self, key: KeyIndex) -> bool {
        match key {
            KeyIndex::K17 | KeyIndex::K18 | KeyIndex::K29 | KeyIndex::K20 | KeyIndex::K48 => {
                self.key_buf.0.is_some() || self.key_buf.1.is_some() || self.key_buf.2.is_some()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.key_buf.0.is_some() || self.key_buf.1.is_some()
    }
}

fn default_or_alt(source: Option<Bopomofo>, default: Bopomofo, alt: Bopomofo) -> Bopomofo {
    match source {
        None => default,
        Some(src) => {
            if src == default {
                alt
            } else {
                default
            }
        }
    }
}

impl PhoneticKeyEditor for DaiChien26 {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_end_key(key.index) {
            let tone = match key.index {
                // KeyIndex::K48 => Some(Bopomofo::TONE1),
                KeyIndex::K17 => Some(Bopomofo::TONE2),
                KeyIndex::K18 => Some(Bopomofo::TONE3),
                KeyIndex::K29 => Some(Bopomofo::TONE4),
                KeyIndex::K20 => Some(Bopomofo::TONE5),
                _ => None,
            };
            self.key_buf.3 = tone;
            return KeyBehavior::Commit;
        }
        let bopomofo = match key.index {
            KeyIndex::K15 => default_or_alt(self.key_buf.0, Bopomofo::B, Bopomofo::P),
            KeyIndex::K27 => Bopomofo::M,
            KeyIndex::K38 => Bopomofo::F,
            KeyIndex::K16 => default_or_alt(self.key_buf.0, Bopomofo::D, Bopomofo::T),
            KeyIndex::K28 => Bopomofo::N,
            KeyIndex::K39 => Bopomofo::L,
            KeyIndex::K17 => Bopomofo::G,
            KeyIndex::K29 => Bopomofo::K,
            KeyIndex::K40 => Bopomofo::H,
            KeyIndex::K18 => Bopomofo::J,
            KeyIndex::K30 => Bopomofo::Q,
            KeyIndex::K41 => Bopomofo::X,
            KeyIndex::K19 => default_or_alt(self.key_buf.0, Bopomofo::ZH, Bopomofo::CH),
            KeyIndex::K31 => Bopomofo::SH,
            KeyIndex::K42 => {
                if self.has_initial_or_medial() {
                    Bopomofo::EH
                } else {
                    Bopomofo::R
                }
            }
            KeyIndex::K20 => Bopomofo::Z,
            KeyIndex::K32 => Bopomofo::C,
            KeyIndex::K43 => {
                if self.has_initial_or_medial() {
                    Bopomofo::ENG
                } else {
                    Bopomofo::S
                }
            }
            KeyIndex::K21 => {
                match (self.key_buf.1, self.key_buf.2) {
                    (Some(Bopomofo::I), Some(Bopomofo::A)) => {
                        self.key_buf.1.take();
                        self.key_buf.2.take();
                        return KeyBehavior::Absorb;
                    }
                    (_, Some(Bopomofo::A)) => {
                        self.key_buf.1.replace(Bopomofo::I);
                        return KeyBehavior::Absorb;
                    }
                    (Some(Bopomofo::I), _) => {
                        self.key_buf.1.take();
                        self.key_buf.2.replace(Bopomofo::A);
                        return KeyBehavior::Absorb;
                    }
                    (Some(_), _) => {
                        self.key_buf.2.replace(Bopomofo::A);
                        return KeyBehavior::Absorb;
                    }
                    _ => (),
                }
                Bopomofo::I
            }
            KeyIndex::K33 => Bopomofo::U,
            KeyIndex::K44 => {
                match (self.key_buf.1, self.key_buf.2) {
                    (Some(Bopomofo::IU), None) => {
                        self.key_buf.1.take();
                        self.key_buf.2.replace(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    (Some(Bopomofo::IU), Some(f)) if f != Bopomofo::OU => {
                        self.key_buf.1.take();
                        self.key_buf.2.replace(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    (None, Some(Bopomofo::OU)) => {
                        self.key_buf.1.replace(Bopomofo::IU);
                        self.key_buf.2.take();
                        return KeyBehavior::Absorb;
                    }
                    (Some(f), Some(Bopomofo::OU)) if f != Bopomofo::IU => {
                        self.key_buf.1.replace(Bopomofo::IU);
                        self.key_buf.2.take();
                        return KeyBehavior::Absorb;
                    }
                    (Some(_), _) => {
                        self.key_buf.2.replace(Bopomofo::OU);
                        return KeyBehavior::Absorb;
                    }
                    _ => (),
                }
                Bopomofo::IU
            }
            KeyIndex::K22 => default_or_alt(self.key_buf.2, Bopomofo::O, Bopomofo::AI),
            KeyIndex::K34 => Bopomofo::E,
            KeyIndex::K23 => default_or_alt(self.key_buf.2, Bopomofo::EI, Bopomofo::AN),
            KeyIndex::K35 => default_or_alt(self.key_buf.2, Bopomofo::AU, Bopomofo::ANG),
            KeyIndex::K24 => default_or_alt(self.key_buf.2, Bopomofo::EN, Bopomofo::ER),
            _ => return KeyBehavior::KeyError,
        };

        match bopomofo.kind() {
            BopomofoKind::Initial => self.key_buf.0.replace(bopomofo),
            BopomofoKind::MedialGlide => self.key_buf.1.replace(bopomofo),
            BopomofoKind::Final => self.key_buf.2.replace(bopomofo),
            BopomofoKind::Tone => self.key_buf.3.replace(bopomofo),
        };

        KeyBehavior::Absorb
    }

    fn is_entering(&self) -> bool {
        !self.key_buf.is_empty()
    }

    fn pop(&mut self) -> Option<Bopomofo> {
        if self.key_buf.3.is_some() {
            return self.key_buf.3.take();
        } else if self.key_buf.2.is_some() {
            return self.key_buf.2.take();
        } else if self.key_buf.1.is_some() {
            return self.key_buf.1.take();
        } else if self.key_buf.0.is_some() {
            return self.key_buf.0.take();
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
