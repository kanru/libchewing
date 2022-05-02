//! Hsu keyboard layout

use crate::{
    bopomofo::{Bopomofo, BopomofoKind},
    keymap::KeyCode,
};

use super::{KeyBehavior, KeyBuf, KeyEvent, PhoneticKeyEditor};

#[derive(Debug)]
pub struct Hsu {
    key_buf: KeyBuf,
}

impl Hsu {
    pub fn new() -> Hsu {
        Hsu {
            key_buf: Default::default(),
        }
    }
    pub fn from_raw_parts(pho_inx: &[i32]) -> Hsu {
        Hsu {
            key_buf: KeyBuf::from_raw_parts(pho_inx),
        }
    }
    fn is_hsu_end_key(&self, key: KeyEvent) -> bool {
        // TODO allow customize end key mapping
        match key.code {
            KeyCode::S | KeyCode::D | KeyCode::F | KeyCode::J | KeyCode::Space => {
                self.key_buf.0.is_some() || self.key_buf.1.is_some() || self.key_buf.2.is_some()
            }
            _ => false,
        }
    }
    fn has_initial_or_medial(&self) -> bool {
        self.key_buf.0.is_some() || self.key_buf.1.is_some()
    }
}

impl PhoneticKeyEditor for Hsu {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if self.is_hsu_end_key(key) {
            if self.key_buf.1.is_none() && self.key_buf.2.is_none() {
                if let Some(key) = self.key_buf.0 {
                    match key {
                        Bopomofo::J => {
                            self.key_buf.0.replace(Bopomofo::ZH);
                        }
                        Bopomofo::Q => {
                            self.key_buf.0.replace(Bopomofo::CH);
                        }
                        Bopomofo::X => {
                            self.key_buf.0.replace(Bopomofo::SH);
                        }
                        Bopomofo::H => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::O);
                        }
                        Bopomofo::G => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::E);
                        }
                        Bopomofo::M => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::AN);
                        }
                        Bopomofo::N => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::EN);
                        }
                        Bopomofo::K => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::ANG);
                        }
                        Bopomofo::L => {
                            self.key_buf.0.take();
                            self.key_buf.2.replace(Bopomofo::ER);
                        }
                        _ => (),
                    }
                }
            }

            // fuzzy ㄍㄧ to ㄐㄧ and ㄍㄩ to ㄐㄩ
            match (self.key_buf.0, self.key_buf.1) {
                (Some(Bopomofo::G), Some(Bopomofo::I)) | (Some(Bopomofo::J), Some(Bopomofo::I)) => {
                    self.key_buf.0.replace(Bopomofo::IU);
                }
                _ => (),
            }

            let tone = match key.code {
                // KeyCode::Space => Some(Bopomofo::TONE1),
                KeyCode::D => Some(Bopomofo::TONE2),
                KeyCode::F => Some(Bopomofo::TONE3),
                KeyCode::J => Some(Bopomofo::TONE4),
                KeyCode::S => Some(Bopomofo::TONE5),
                _ => None,
            };
            self.key_buf.3 = tone;
            KeyBehavior::Commit
        } else {
            let bopomofo = match key.code {
                KeyCode::A => {
                    if self.has_initial_or_medial() {
                        Bopomofo::EI
                    } else {
                        Bopomofo::C
                    }
                }
                KeyCode::B => Bopomofo::B,
                KeyCode::C => Bopomofo::SH,
                KeyCode::D => Bopomofo::D,
                KeyCode::E => Bopomofo::I,
                KeyCode::F => Bopomofo::F,
                KeyCode::G => {
                    if self.has_initial_or_medial() {
                        Bopomofo::E
                    } else {
                        Bopomofo::G
                    }
                }
                KeyCode::H => {
                    if self.has_initial_or_medial() {
                        Bopomofo::O
                    } else {
                        Bopomofo::H
                    }
                }
                KeyCode::I => Bopomofo::AI,
                KeyCode::J => Bopomofo::ZH,
                KeyCode::K => {
                    if self.has_initial_or_medial() {
                        Bopomofo::ANG
                    } else {
                        Bopomofo::K
                    }
                }
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
                KeyCode::O => Bopomofo::OU,
                KeyCode::P => Bopomofo::P,
                KeyCode::R => Bopomofo::R,
                KeyCode::S => Bopomofo::S,
                KeyCode::T => Bopomofo::T,
                KeyCode::U => Bopomofo::IU,
                KeyCode::V => Bopomofo::CH,
                KeyCode::W => Bopomofo::AU,
                KeyCode::X => Bopomofo::U,
                KeyCode::Y => Bopomofo::A,
                KeyCode::Z => Bopomofo::Z,
                _ => return KeyBehavior::NoWord,
            };
            let kind = bopomofo.kind();

            // fuzzy ㄍㄧ to ㄐㄧ and ㄍㄩ to ㄐㄩ
            match (self.key_buf.0, self.key_buf.1) {
                (Some(Bopomofo::G), Some(Bopomofo::I)) | (Some(Bopomofo::J), Some(Bopomofo::I)) => {
                    self.key_buf.0.replace(Bopomofo::IU);
                }
                _ => (),
            }

            // ㄐㄑㄒ must be followed by ㄧ or ㄩ. If not, convert them to ㄓㄔㄕ
            if (kind == BopomofoKind::MedialGlide && bopomofo == Bopomofo::U)
                || (kind == BopomofoKind::Final && self.key_buf.1.is_none())
            {
                match self.key_buf.0 {
                    Some(Bopomofo::J) => {
                        self.key_buf.0.replace(Bopomofo::ZH);
                    }
                    Some(Bopomofo::Q) => {
                        self.key_buf.0.replace(Bopomofo::CH);
                    }
                    Some(Bopomofo::X) => {
                        self.key_buf.0.replace(Bopomofo::SH);
                    }
                    _ => (),
                }
            }

            // Likeweise, when ㄓㄔㄕ is followed by ㄧ or ㄩ, convert them to ㄐㄑㄒ
            if bopomofo == Bopomofo::I || bopomofo == Bopomofo::IU {
                match self.key_buf.0 {
                    Some(Bopomofo::ZH) => {
                        self.key_buf.0.replace(Bopomofo::J);
                    }
                    Some(Bopomofo::CH) => {
                        self.key_buf.0.replace(Bopomofo::Q);
                    }
                    Some(Bopomofo::SH) => {
                        self.key_buf.0.replace(Bopomofo::X);
                    }
                    _ => (),
                }
            }

            match kind {
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

#[cfg(test)]
mod test {

    use crate::{
        bopomofo::Bopomofo,
        editor::phonetic::PhoneticKeyEditor,
        keymap::{IdentityKeymap, KeyCode, Keymap, QWERTY},
    };

    use super::Hsu;

    #[test]
    fn cen() {
        let mut hsu = Hsu::new();
        let keymap = IdentityKeymap::new(QWERTY);
        hsu.key_press(keymap.map_key(KeyCode::C));
        hsu.key_press(keymap.map_key(KeyCode::E));
        hsu.key_press(keymap.map_key(KeyCode::N));
        hsu.key_press(keymap.map_key(KeyCode::Space));
        let result = hsu.observe();
        assert_eq!(result.0, Some(Bopomofo::X));
        assert_eq!(result.1, Some(Bopomofo::I));
        assert_eq!(result.2, Some(Bopomofo::EN));
    }

    #[test]
    fn convert_n_to_en() {
        let mut hsu = Hsu::new();
        let keymap = IdentityKeymap::new(QWERTY);
        hsu.key_press(keymap.map_key(KeyCode::N));
        hsu.key_press(keymap.map_key(KeyCode::F));
        let result = hsu.observe();
        assert_eq!(result.2, Some(Bopomofo::EN));
    }
}
