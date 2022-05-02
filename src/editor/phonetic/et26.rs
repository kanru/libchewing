//! ET26 (倚天26鍵)

use crate::{
    bopomofo::{Bopomofo, BopomofoKind},
    keymap::{KeyCode, KeyEvent},
};

use super::{KeyBehavior, KeyBuf, PhoneticKeyEditor};

#[derive(Debug)]
pub struct Et26 {
    key_buf: KeyBuf,
}

impl Et26 {
    pub fn new() -> Et26 {
        Et26 {
            key_buf: Default::default(),
        }
    }
    pub fn from_raw_parts(pho_inx: &[i32]) -> Et26 {
        Et26 {
            key_buf: KeyBuf::from_raw_parts(pho_inx),
        }
    }
    fn is_end_key(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::D | KeyCode::F | KeyCode::J | KeyCode::K | KeyCode::Space => {
                self.key_buf.0.is_some() || self.key_buf.1.is_some() || self.key_buf.2.is_some()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.key_buf.0.is_some() || self.key_buf.1.is_some()
    }
}

impl PhoneticKeyEditor for Et26 {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_end_key(key.code) {
            if self.key_buf.1.is_none() && self.key_buf.2.is_none() {
                match self.key_buf.0 {
                    Some(Bopomofo::J) => {
                        self.key_buf.0.replace(Bopomofo::ZH);
                    }
                    Some(Bopomofo::X) => {
                        self.key_buf.0.replace(Bopomofo::SH);
                    }
                    Some(Bopomofo::P) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::OU);
                    }
                    Some(Bopomofo::M) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::AN);
                    }
                    Some(Bopomofo::N) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::EN);
                    }
                    Some(Bopomofo::T) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::ANG);
                    }
                    Some(Bopomofo::L) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::ENG);
                    }
                    Some(Bopomofo::H) => {
                        self.key_buf.0.take();
                        self.key_buf.2.replace(Bopomofo::ER);
                    }
                    _ => (),
                }
            }
            let tone = match key.code {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                KeyCode::F => Some(Bopomofo::TONE2),
                KeyCode::J => Some(Bopomofo::TONE3),
                KeyCode::K => Some(Bopomofo::TONE4),
                KeyCode::D => Some(Bopomofo::TONE5),
                _ => None,
            };
            self.key_buf.3 = tone;
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.code {
                KeyCode::A => Bopomofo::A,
                KeyCode::B => Bopomofo::B,
                KeyCode::C => Bopomofo::X,
                KeyCode::D => Bopomofo::D,
                KeyCode::E => Bopomofo::I,
                KeyCode::F => Bopomofo::F,
                KeyCode::G => Bopomofo::J,
                KeyCode::H => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ER
                    } else {
                        Bopomofo::H
                    }
                }
                KeyCode::I => Bopomofo::AI,
                KeyCode::J => Bopomofo::R,
                KeyCode::K => Bopomofo::K,
                KeyCode::L => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ENG
                    } else {
                        Bopomofo::L
                    }
                }
                KeyCode::M => {
                    if self.has_initial_or_medial() {
                        Bopomofo::AN
                    } else {
                        Bopomofo::M
                    }
                }
                KeyCode::N => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EN
                    } else {
                        Bopomofo::N
                    }
                }
                KeyCode::O => Bopomofo::O,
                KeyCode::P => {
                    if self.has_initial_or_medial() {
                        Bopomofo::OU
                    } else {
                        Bopomofo::P
                    }
                }
                KeyCode::Q => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::Z
                    }
                }
                KeyCode::R => Bopomofo::E,
                KeyCode::S => Bopomofo::S,
                KeyCode::T => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::T
                    }
                }
                KeyCode::U => Bopomofo::IU,
                KeyCode::V => Bopomofo::G,
                KeyCode::W => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EH
                    } else {
                        Bopomofo::C
                    }
                }
                KeyCode::X => Bopomofo::U,
                KeyCode::Y => Bopomofo::CH,
                KeyCode::Z => Bopomofo::AU,
                _ => return KeyBehavior::NoWord,
            };

            match bopomofo.kind() {
                BopomofoKind::MedialGlide => {
                    if bopomofo == Bopomofo::U {
                        match self.key_buf.0 {
                            Some(Bopomofo::J) => {
                                self.key_buf.0.replace(Bopomofo::ZH);
                            }
                            Some(Bopomofo::X) => {
                                self.key_buf.0.replace(Bopomofo::SH);
                            }
                            _ => (),
                        }
                    } else if let Some(Bopomofo::G) = self.key_buf.0 {
                        self.key_buf.0.replace(Bopomofo::Q);
                    }
                }
                BopomofoKind::Final if self.key_buf.1.is_none() => {
                    match self.key_buf.0 {
                        Some(Bopomofo::J) => {
                            self.key_buf.0.replace(Bopomofo::ZH);
                        }
                        Some(Bopomofo::X) => {
                            self.key_buf.0.replace(Bopomofo::SH);
                        }
                        _ => (),
                    };
                }
                _ => (),
            };

            match bopomofo.kind() {
                BopomofoKind::Initial => self.key_buf.0.replace(bopomofo),
                BopomofoKind::MedialGlide => self.key_buf.1.replace(bopomofo),
                BopomofoKind::Final => self.key_buf.2.replace(bopomofo),
                BopomofoKind::Tone => self.key_buf.3.replace(bopomofo),
            };

            KeyBehavior::Absorb
        }
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
