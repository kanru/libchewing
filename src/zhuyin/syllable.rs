use std::fmt::{Display, Write};

use miette::Diagnostic;
use thiserror::Error;

use super::{Bopomofo, BopomofoKind};

/// The consonants and vowels that are taken together to make a single sound.
///
/// <https://en.m.wikipedia.org/wiki/Syllable#Chinese_model>
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Syllable {
    pub initial: Option<Bopomofo>,
    pub medial: Option<Bopomofo>,
    pub rime: Option<Bopomofo>,
    pub tone: Option<Bopomofo>,
}

impl Syllable {
    pub const fn new() -> Syllable {
        Syllable {
            initial: None,
            medial: None,
            rime: None,
            tone: None,
        }
    }

    pub const fn builder() -> SyllableBuilder {
        SyllableBuilder {
            syllable: Syllable::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.initial.is_none()
            && self.medial.is_none()
            && self.rime.is_none()
            && self.tone.is_none()
    }
    pub fn has_initial(&self) -> bool {
        self.initial.is_some()
    }
    pub fn has_medial(&self) -> bool {
        self.medial.is_some()
    }
    pub fn has_rime(&self) -> bool {
        self.rime.is_some()
    }
    pub fn has_tone(&self) -> bool {
        self.tone.is_some()
    }
    pub fn as_u16(&self) -> u16 {
        assert!(
            !self.is_empty(),
            "empty syllable cannot be converted to u16"
        );
        let initial = self.initial.map_or(0, |v| v as u16 + 1);
        let medial = self.medial.map_or(0, |v| v as u16 - 20);
        let rime = self.rime.map_or(0, |v| v as u16 - 23);
        let tone = self.tone.map_or(0, |v| v as u16 - 36).clamp(0, 4);

        (initial << 9) | (medial << 7) | (rime << 3) | tone
    }
    pub fn from_u16(_syllable: u16) -> Result<Syllable, SyllableDecodeError> {
        todo!()
    }
    pub fn update(&mut self, bopomofo: Bopomofo) {
        match bopomofo.kind() {
            BopomofoKind::Initial => self.initial.replace(bopomofo),
            BopomofoKind::Medial => self.medial.replace(bopomofo),
            BopomofoKind::Rime => self.rime.replace(bopomofo),
            BopomofoKind::Tone => self.tone.replace(bopomofo),
        };
    }
    pub fn pop(&mut self) -> Option<Bopomofo> {
        for bopomofo in [
            &mut self.tone,
            &mut self.rime,
            &mut self.medial,
            &mut self.initial,
        ] {
            if bopomofo.is_some() {
                return bopomofo.take();
            }
        }
        None
    }
    pub fn clear(&mut self) {
        *self = Syllable::new()
    }
}

impl Default for Syllable {
    fn default() -> Self {
        Syllable::new()
    }
}

impl Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &bopomofo in [&self.initial, &self.medial, &self.rime, &self.tone] {
            if let Some(bopomofo) = bopomofo {
                f.write_char(bopomofo.into())?;
            }
        }
        Ok(())
    }
}

pub struct SyllableBuilder {
    syllable: Syllable,
}

impl SyllableBuilder {
    pub const fn insert(mut self, bopomofo: Bopomofo) -> SyllableBuilder {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                if self.syllable.initial.is_some() {
                    panic!("multiple initial bopomofo");
                }
                self.syllable.initial = Some(bopomofo);
            }
            BopomofoKind::Medial => {
                if self.syllable.medial.is_some() {
                    panic!("multiple medial bopomofo");
                }
                self.syllable.medial = Some(bopomofo);
            }
            BopomofoKind::Rime => {
                if self.syllable.rime.is_some() {
                    panic!("multiple rime bopomofo");
                }
                self.syllable.rime = Some(bopomofo);
            }
            BopomofoKind::Tone => {
                if self.syllable.tone.is_some() {
                    panic!("multiple tone bopomofo");
                }
                self.syllable.tone = Some(bopomofo);
            }
        };
        self
    }
    pub const fn build(self) -> Syllable {
        self.syllable
    }
}
#[derive(Error, Diagnostic, Debug)]
#[error("syllable decode error: {msg}")]
#[diagnostic(code(chewing::syllable_decode_error))]
pub struct SyllableDecodeError {
    msg: String,
}

#[macro_export]
macro_rules! syl {
    () => { $crate::zhuyin::Syllable::new() };
    ($($bopomofo:expr),+) => {
        {
            let mut builder = $crate::zhuyin::Syllable::builder();
            $(builder = builder.insert($bopomofo);)+
            builder.build()
        }
    };
}

#[cfg(test)]
mod test {

    use super::{Bopomofo, Syllable};

    #[test]
    fn syllable_hsu_sdf_as_u16() {
        let syl = Syllable::builder().insert(Bopomofo::S).build();
        assert_eq!(0x2A00, syl.as_u16());

        let syl = Syllable::builder().insert(Bopomofo::D).build();
        assert_eq!(0xA00, syl.as_u16());

        let syl = Syllable::builder().insert(Bopomofo::F).build();
        assert_eq!(0x800, syl.as_u16());
    }

    #[test]
    #[should_panic]
    fn empty_syllable_as_u16() {
        Syllable::builder().build().as_u16();
    }

    #[test]
    fn syl_macro_rules() {
        let syl = syl![];
        assert_eq!(Syllable::new(), syl);

        let syl = syl![Bopomofo::S];
        assert_eq!(Syllable::builder().insert(Bopomofo::S).build(), syl);

        let syl = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4];
        assert_eq!(
            Syllable::builder()
                .insert(Bopomofo::S)
                .insert(Bopomofo::I)
                .insert(Bopomofo::EN)
                .insert(Bopomofo::TONE4)
                .build(),
            syl
        );
    }

    #[test]
    #[should_panic]
    fn syl_macro_rules_fool_proof() {
        syl![Bopomofo::S, Bopomofo::D];
    }

    #[test]
    fn syl_macro_rules_comiles_in_const() {
        const SYLLABLE: Syllable = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN];
        assert_eq!(
            Syllable::builder()
                .insert(Bopomofo::S)
                .insert(Bopomofo::I)
                .insert(Bopomofo::EN)
                .build(),
            SYLLABLE
        );
    }

    #[test]
    fn new_and_pop_bopomofo() {
        let mut syl = syl![Bopomofo::S, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE4];
        assert_eq!(Some(Bopomofo::TONE4), syl.pop());
        assert_eq!(Some(Bopomofo::EN), syl.pop());
        assert_eq!(Some(Bopomofo::I), syl.pop());
        assert_eq!(Some(Bopomofo::S), syl.pop());
        assert_eq!(None, syl.pop());
        assert_eq!(syl![], syl);
    }
}
