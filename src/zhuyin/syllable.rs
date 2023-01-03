use std::fmt::{Display, Write};

use thiserror::Error;

use super::{Bopomofo, BopomofoKind};

/// The consonants and vowels that are taken together to make a single sound.
///
/// <https://en.m.wikipedia.org/wiki/Syllable#Chinese_model>
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Syllable {
    value: u16,
}

impl Syllable {
    pub const fn new() -> Syllable {
        Syllable {
            value: 0,
        }
    }

    pub const fn builder() -> SyllableBuilder {
        SyllableBuilder {
            value: 0,
        }
    }
    pub const fn initial(&self) -> Option<Bopomofo> {
        let index = self.value >> 9;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_initial(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    pub const fn medial(&self) -> Option<Bopomofo> {
        let index = (self.value & 0b0000000_11_0000_000) >> 7;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_medial(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    pub const fn rime(&self) -> Option<Bopomofo> {
        let index = (self.value & 0b0000000_00_1111_000) >> 3;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_rime(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    pub const fn tone(&self) -> Option<Bopomofo> {
        let index = self.value & 0b0000000_00_0000_111;
        if index == 0 {
            None
        } else {
            match Bopomofo::from_tone(index) {
                Ok(v) => Some(v),
                Err(_) => panic!(),
            }
        }
    }
    pub fn remove_initial(&mut self) -> Option<Bopomofo> {
        let ret = self.initial();
        self.value = 0b0000000_11_1111_111 & self.value;
        ret
    }
    pub fn remove_medial(&mut self) -> Option<Bopomofo> {
        let ret = self.medial();
        self.value = 0b1111111_00_1111_111 & self.value;
        ret
    }
    pub fn remove_rime(&mut self) -> Option<Bopomofo> {
        let ret = self.rime();
        self.value = 0b1111111_11_0000_111 & self.value;
        ret
    }
    pub fn remove_tone(&mut self) -> Option<Bopomofo> {
        let ret = self.tone();
        self.value = 0b1111111_11_1111_000 & self.value;
        ret
    }
    pub fn is_empty(&self) -> bool {
        self.value == 0
    }
    pub fn has_initial(&self) -> bool {
        self.initial().is_some()
    }
    pub fn has_medial(&self) -> bool {
        self.medial().is_some()
    }
    pub fn has_rime(&self) -> bool {
        self.rime().is_some()
    }
    pub fn has_tone(&self) -> bool {
        self.tone().is_some()
    }
    /// Returns the `Syllable` encoded in a u16 integer.
    ///
    /// The data layout used:
    ///
    /// ```text
    ///  0                   1
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |   Initial   | M | Rime  |Tone |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    pub fn to_u16(&self) -> u16 {
        debug_assert!(
            !self.is_empty(),
            "empty syllable cannot be converted to u16"
        );
        self.value
    }
    /// Returns the `Syllable` encoded in a u16 integer in little-endian bytes.
    ///
    /// The data layout used:
    ///
    /// ```text
    ///  0                   1
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |   Initial   | M | Rime  |Tone |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.to_u16().to_le_bytes()
    }
    pub fn update(&mut self, bopomofo: Bopomofo) {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                self.remove_initial();
                self.value |= (bopomofo as u16 + 1) << 9;
            }
            BopomofoKind::Medial => {
                self.remove_medial();
                self.value |= (bopomofo as u16 - 20) << 7;
            }
            BopomofoKind::Rime => {
                self.remove_rime();
                self.value |= (bopomofo as u16 - 23) << 3;
            }
            BopomofoKind::Tone => {
                self.remove_tone();
                self.value |= bopomofo as u16 - 36;
            }
        };
    }
    pub fn pop(&mut self) -> Option<Bopomofo> {
        if self.tone().is_some() {
            return self.remove_tone();
        }
        if self.rime().is_some() {
            return self.remove_rime();
        }
        if self.medial().is_some() {
            return self.remove_medial();
        }
        if self.initial().is_some() {
            return self.remove_initial();
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

impl From<Syllable> for u16 {
    fn from(syl: Syllable) -> Self {
        syl.to_u16()
    }
}

impl From<&Syllable> for u16 {
    fn from(syl: &Syllable) -> Self {
        syl.to_u16()
    }
}

impl TryFrom<u16> for Syllable {
    type Error = DecodeSyllableError;

    #[allow(clippy::unusual_byte_groupings)]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // TODO check invalid value
        Ok(Syllable {
            value,
        })
    }
}

pub trait IntoSyllablesBytes {
    fn into_syllables_bytes(self) -> Vec<u8>;
}

impl<T> IntoSyllablesBytes for T
where
    T: IntoIterator,
    T::Item: Into<u16>,
{
    fn into_syllables_bytes(self) -> Vec<u8> {
        let mut syllables_bytes = vec![];
        self.into_iter()
            .for_each(|syl| syllables_bytes.extend_from_slice(&syl.into().to_le_bytes()));
        syllables_bytes
    }
}

impl Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &bopomofo in [&self.initial(), &self.medial(), &self.rime(), &self.tone()] {
            if let Some(bopomofo) = bopomofo {
                f.write_char(bopomofo.into())?;
            }
        }
        Ok(())
    }
}

pub struct SyllableBuilder {
    value: u16,
}

impl SyllableBuilder {
    pub const fn insert(mut self, bopomofo: Bopomofo) -> SyllableBuilder {
        match bopomofo.kind() {
            BopomofoKind::Initial => {
                if self.value & 0b1111111_00_0000_000 != 0 {
                    panic!("multiple initial bopomofo");
                }
                self.value |= (bopomofo as u16 + 1) << 9;
            }
            BopomofoKind::Medial => {
                if self.value & 0b0000000_11_0000_000 != 0 {
                    panic!("multiple medial bopomofo");
                }
                self.value |= (bopomofo as u16 - 20) << 7;
            }
            BopomofoKind::Rime => {
                if self.value & 0b0000000_00_1111_000 != 0 {
                    panic!("multiple rime bopomofo");
                }
                self.value |= (bopomofo as u16 - 23) << 3;
            }
            BopomofoKind::Tone => {
                if self.value & 0b0000000_00_0000_111 != 0 {
                    panic!("multiple tone bopomofo");
                }
                self.value |= bopomofo as u16 - 36;
            }
        };
        self
    }
    pub const fn build(self) -> Syllable {
        Syllable { value: self.value }
    }
}
#[derive(Error, Debug)]
#[error("syllable decode error: {msg}")]
pub struct DecodeSyllableError {
    msg: String,
    source: Box<dyn std::error::Error>,
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
        assert_eq!(0x2A00, syl.to_u16());

        let syl = Syllable::builder().insert(Bopomofo::D).build();
        assert_eq!(0xA00, syl.to_u16());

        let syl = Syllable::builder().insert(Bopomofo::F).build();
        assert_eq!(0x800, syl.to_u16());
    }

    #[test]
    #[should_panic]
    fn empty_syllable_as_u16() {
        Syllable::builder().build().to_u16();
    }

    #[test]
    fn syllable_as_u16_roundtrip() {
        let syl = Syllable::builder().insert(Bopomofo::S).build();
        assert_eq!(syl, syl.to_u16().try_into().unwrap());
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
