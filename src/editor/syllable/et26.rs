//! ET26 (倚天26鍵)

use crate::{
    keymap::{KeyCode, KeyEvent},
    zhuyin::{Bopomofo, BopomofoKind, Syllable},
};

use super::{KeyBehavior, SyllableEditor};

#[derive(Debug)]
pub struct Et26 {
    syllable: Syllable,
}

impl Et26 {
    pub fn new() -> Et26 {
        Et26 {
            syllable: Default::default(),
        }
    }
    fn is_end_key(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::D | KeyCode::F | KeyCode::J | KeyCode::K | KeyCode::Space => {
                !self.syllable.is_empty()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.syllable.has_initial() || self.syllable.has_medial()
    }
}

impl SyllableEditor for Et26 {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_end_key(key.code) {
            if !self.syllable.has_medial() && !self.syllable.has_rime() {
                match self.syllable.initial {
                    Some(Bopomofo::J) => {
                        self.syllable.update(Bopomofo::ZH);
                    }
                    Some(Bopomofo::X) => {
                        self.syllable.update(Bopomofo::SH);
                    }
                    Some(Bopomofo::P) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::OU);
                    }
                    Some(Bopomofo::M) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::AN);
                    }
                    Some(Bopomofo::N) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::EN);
                    }
                    Some(Bopomofo::T) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::ANG);
                    }
                    Some(Bopomofo::L) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::ENG);
                    }
                    Some(Bopomofo::H) => {
                        self.syllable.initial.take();
                        self.syllable.update(Bopomofo::ER);
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
            self.syllable.tone = tone;
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
                BopomofoKind::Medial => {
                    if bopomofo == Bopomofo::U {
                        match self.syllable.initial {
                            Some(Bopomofo::J) => {
                                self.syllable.update(Bopomofo::ZH);
                            }
                            Some(Bopomofo::X) => {
                                self.syllable.update(Bopomofo::SH);
                            }
                            _ => (),
                        }
                    } else if let Some(Bopomofo::G) = self.syllable.initial {
                        self.syllable.update(Bopomofo::Q);
                    }
                }
                BopomofoKind::Rime if !self.syllable.has_medial() => {
                    match self.syllable.initial {
                        Some(Bopomofo::J) => {
                            self.syllable.update(Bopomofo::ZH);
                        }
                        Some(Bopomofo::X) => {
                            self.syllable.update(Bopomofo::SH);
                        }
                        _ => (),
                    };
                }
                _ => (),
            };

            self.syllable.update(bopomofo);
            KeyBehavior::Absorb
        }
    }

    fn is_entering(&self) -> bool {
        !self.syllable.is_empty()
    }

    fn pop(&mut self) -> Option<Bopomofo> {
        self.syllable.pop()
    }

    fn clear(&mut self) {
        self.syllable.clear();
    }

    fn observe(&self) -> Syllable {
        self.syllable
    }
}
