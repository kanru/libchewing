//! Pinyin

use std::ffi::CStr;

use crate::{
    bopomofo::Bopomofo,
    keymap::{KeyCode, KeyEvent},
};

use super::{KeyBehavior, KeyBuf, PhoneticKeyEditor};

const MAX_PINYIN_LEN: usize = 10;

pub enum PinyinVariant {
    HanyuPinyin,
    ThlPinyin,
    Mps2Pinyin,
}

impl Default for PinyinVariant {
    fn default() -> PinyinVariant {
        PinyinVariant::HanyuPinyin
    }
}

#[derive(Default)]
pub struct Pinyin {
    key_seq: String,
    key_buf: KeyBuf,
    key_buf_alt: KeyBuf,
    variant: PinyinVariant,
}

impl Pinyin {
    pub fn new() -> Pinyin {
        Default::default()
    }
    pub fn from_raw_parts(
        kb_type: PinyinVariant,
        raw_seq: &CStr,
        pho_inx: &[i32],
        pho_inx_alt: &[i32],
    ) -> Pinyin {
        dbg!(&raw_seq);
        Pinyin {
            key_seq: raw_seq.to_owned().into_string().unwrap(),
            key_buf: KeyBuf::from_raw_parts(pho_inx),
            key_buf_alt: KeyBuf::from_raw_parts(pho_inx_alt),
            variant: kb_type,
        }
    }
    pub fn alt(&self) -> KeyBuf {
        self.key_buf_alt
    }
    pub fn key_seq(&self) -> &String {
        &self.key_seq
    }
}

impl PhoneticKeyEditor for Pinyin {
    fn key_press(&mut self, key: KeyEvent) -> KeyBehavior {
        if ![
            KeyCode::Space,
            KeyCode::N1,
            KeyCode::N2,
            KeyCode::N3,
            KeyCode::N4,
            KeyCode::N5,
        ]
        .contains(&key.code)
        {
            if self.key_seq.len() == MAX_PINYIN_LEN {
                // buffer is full, ignore this keystroke
                return KeyBehavior::NoWord;
            }
            let ch = match key.code {
                KeyCode::A => 'a',
                KeyCode::B => 'b',
                KeyCode::C => 'c',
                KeyCode::D => 'd',
                KeyCode::E => 'e',
                KeyCode::F => 'f',
                KeyCode::G => 'g',
                KeyCode::H => 'h',
                KeyCode::I => 'i',
                KeyCode::J => 'j',
                KeyCode::K => 'k',
                KeyCode::L => 'l',
                KeyCode::M => 'm',
                KeyCode::N => 'n',
                KeyCode::O => 'o',
                KeyCode::P => 'p',
                KeyCode::Q => 'q',
                KeyCode::R => 'r',
                KeyCode::S => 's',
                KeyCode::T => 't',
                KeyCode::U => 'u',
                KeyCode::V => 'v',
                KeyCode::W => 'w',
                KeyCode::X => 'x',
                KeyCode::Y => 'y',
                KeyCode::Z => 'z',
                _ => return KeyBehavior::KeyError,
            };
            self.key_seq.push(ch);
            return KeyBehavior::Absorb;
        }

        if let Some(entry) = match self.variant {
            PinyinVariant::HanyuPinyin => HANYU_PINYIN_MAPPING.iter(),
            PinyinVariant::ThlPinyin => THL_PINYIN_MAPPING.iter(),
            PinyinVariant::Mps2Pinyin => MPS2_PINYIN_MAPPING.iter(),
        }
        .find(|entry| entry.pinyin == self.key_seq)
        {
            self.key_seq.clear();
            self.key_buf = entry.primary;
            self.key_buf_alt = entry.alt;
            return KeyBehavior::TryCommit;
        }

        if let Some(entry) = COMMON_MAPPING
            .iter()
            .find(|entry| entry.pinyin == self.key_seq)
        {
            self.key_seq.clear();
            self.key_buf = entry.primary;
            self.key_buf_alt = entry.alt;
            return KeyBehavior::TryCommit;
        }

        let initial = INITIAL_MAPPING
            .iter()
            .find(|entry| self.key_seq.starts_with(&entry.pinyin));

        let final_seq = match initial {
            Some(entry) => self.key_seq.trim_start_matches(&entry.pinyin),
            None => &self.key_seq,
        };

        let fina = FINAL_MAPPING.iter().find(|entry| final_seq == entry.pinyin);

        if initial.is_none() && fina.is_none() {
            self.key_seq.clear();
            return KeyBehavior::Absorb;
        }

        let mut ini = initial.map(|i| i.initial);
        let mut med = fina.and_then(|f| f.medial);
        let mut fin = fina.and_then(|f| f.fina);

        if let Some(Bopomofo::I) = fin {
            match ini {
                Some(Bopomofo::ZH) | Some(Bopomofo::CH) | Some(Bopomofo::SH)
                | Some(Bopomofo::R) | Some(Bopomofo::Z) | Some(Bopomofo::C) | Some(Bopomofo::S) => {
                    med.take();
                    fin.take();
                }
                _ => (),
            }
        }

        match ini {
            Some(Bopomofo::J) | Some(Bopomofo::Q) | Some(Bopomofo::X) => {
                match (med, fin) {
                    (Some(Bopomofo::U), Some(Bopomofo::AN))
                    | (Some(Bopomofo::U), Some(Bopomofo::EN))
                    | (Some(Bopomofo::U), None) => {
                        med.replace(Bopomofo::IU);
                    }
                    _ => (),
                };
            }
            _ => (),
        }

        match med {
            Some(Bopomofo::I) | Some(Bopomofo::IU) => {
                match ini {
                    Some(Bopomofo::S) | Some(Bopomofo::SH) => {
                        ini.replace(Bopomofo::X);
                    }
                    Some(Bopomofo::C) | Some(Bopomofo::CH) => {
                        ini.replace(Bopomofo::Q);
                    }
                    _ => (),
                };
            }
            _ => {
                if ini == Some(Bopomofo::J) {
                    ini.replace(Bopomofo::ZH);
                }
            }
        }

        match ini {
            Some(Bopomofo::B) | Some(Bopomofo::P) | Some(Bopomofo::M) | Some(Bopomofo::F) => {
                match (med, fin) {
                    (Some(Bopomofo::U), Some(Bopomofo::ENG))
                    | (Some(Bopomofo::U), Some(Bopomofo::O)) => {
                        med.take();
                    }
                    _ => (),
                };
            }
            _ => (),
        }

        self.key_seq.clear();
        self.key_buf = KeyBuf(ini, med, fin, None);
        self.key_buf_alt = KeyBuf(ini, med, fin, None);
        KeyBehavior::TryCommit
    }

    fn pop(&mut self) -> Option<Bopomofo> {
        todo!()
    }

    fn clear(&mut self) {
        self.key_seq.clear();
        self.key_buf = Default::default();
        self.key_buf_alt = Default::default();
    }

    fn observe(&self) -> KeyBuf {
        self.key_buf
    }
}

struct AmbiguousMapEntry {
    pinyin: &'static str,
    primary: KeyBuf,
    alt: KeyBuf,
}

macro_rules! amb {
    ($pinyin:expr, { $pi:expr, $pm:expr, $pf:expr }, { $ai:expr, $am:expr, $af:expr } ) => {
        AmbiguousMapEntry {
            pinyin: $pinyin,
            primary: KeyBuf($pi, $pm, $pf, None),
            alt: KeyBuf($ai, $am, $af, None),
        }
    };
}

struct InitialMapEntry {
    pinyin: &'static str,
    initial: Bopomofo,
}

macro_rules! ini {
    ($pinyin:expr, $bopomofo:expr) => {
        InitialMapEntry {
            pinyin: $pinyin,
            initial: $bopomofo,
        }
    };
}

struct FinalMapEntry {
    pinyin: &'static str,
    medial: Option<Bopomofo>,
    fina: Option<Bopomofo>,
}

macro_rules! fin {
    ($pinyin:expr, $medial:expr, $fina:expr) => {
        FinalMapEntry {
            pinyin: $pinyin,
            medial: $medial,
            fina: $fina,
        }
    };
}

const COMMON_MAPPING: [AmbiguousMapEntry; 18] = [
    // Special cases for WG
    amb!("tzu", { Some(Bopomofo::Z), None, None }, { Some(Bopomofo::Z), Some(Bopomofo::U), None }),
    amb!("ssu", { Some(Bopomofo::S), None, None }, { Some(Bopomofo::S), Some(Bopomofo::U), None }),
    amb!("szu", { Some(Bopomofo::S), None, None }, { Some(Bopomofo::S), Some(Bopomofo::U), None }),
    // Common multiple mapping
    amb!("e", { Some(Bopomofo::E), None, None }, { Some(Bopomofo::EH), None, None }),
    amb!("ch", { Some(Bopomofo::CH), None, None }, { Some(Bopomofo::Q), None, None }),
    amb!("sh", { Some(Bopomofo::SH), None, None }, { Some(Bopomofo::X), None, None }),
    amb!("c", { Some(Bopomofo::C), None, None }, { Some(Bopomofo::Q), None, None }),
    amb!("s", { Some(Bopomofo::S), None, None }, { Some(Bopomofo::X), None, None }),
    amb!("nu", { Some(Bopomofo::N), Some(Bopomofo::U), None }, { Some(Bopomofo::N), Some(Bopomofo::IU), None }),
    amb!("lu", { Some(Bopomofo::L), Some(Bopomofo::U), None }, { Some(Bopomofo::L), Some(Bopomofo::IU), None }),
    amb!("luan", { Some(Bopomofo::L), Some(Bopomofo::U), Some(Bopomofo::AN) }, { Some(Bopomofo::L), Some(Bopomofo::IU), Some(Bopomofo::AN) }),
    amb!("niu", { Some(Bopomofo::N), Some(Bopomofo::I), Some(Bopomofo::OU) }, { Some(Bopomofo::N), Some(Bopomofo::IU), None }),
    amb!("liu", { Some(Bopomofo::L), Some(Bopomofo::I), Some(Bopomofo::OU) }, { Some(Bopomofo::L), Some(Bopomofo::IU), None }),
    amb!("jiu", { Some(Bopomofo::J), Some(Bopomofo::I), Some(Bopomofo::OU) }, { Some(Bopomofo::J), Some(Bopomofo::IU), None }),
    amb!("chiu", { Some(Bopomofo::Q), Some(Bopomofo::I), Some(Bopomofo::OU) }, { Some(Bopomofo::Q), Some(Bopomofo::IU), None }),
    amb!("shiu", { Some(Bopomofo::X), Some(Bopomofo::I), Some(Bopomofo::OU) }, { Some(Bopomofo::X), Some(Bopomofo::IU), None }),
    amb!("ju", { Some(Bopomofo::J), Some(Bopomofo::IU), None }, { Some(Bopomofo::ZH), Some(Bopomofo::U), None }),
    amb!("juan", { Some(Bopomofo::J), Some(Bopomofo::IU), Some(Bopomofo::AN) }, { Some(Bopomofo::ZH), Some(Bopomofo::U), Some(Bopomofo::AN) }),
];

const HANYU_PINYIN_MAPPING: [AmbiguousMapEntry; 4] = [
    amb!("chi", { Some(Bopomofo::CH), None, None }, { Some(Bopomofo::Q), Some(Bopomofo::I), None }),
    amb!("shi", { Some(Bopomofo::SH), None, None }, { Some(Bopomofo::X), Some(Bopomofo::I), None }),
    amb!("ci", { Some(Bopomofo::C), None, None }, { Some(Bopomofo::Q), Some(Bopomofo::I), None }),
    amb!("si", { Some(Bopomofo::S), None, None }, { Some(Bopomofo::X), Some(Bopomofo::I), None }),
];

const THL_PINYIN_MAPPING: [AmbiguousMapEntry; 4] = [
    amb!("chi", { Some(Bopomofo::Q), Some(Bopomofo::I), None }, { Some(Bopomofo::CH), None, None }),
    amb!("shi", { Some(Bopomofo::X), Some(Bopomofo::I), None }, { Some(Bopomofo::SH), None, None }),
    amb!("ci", { Some(Bopomofo::Q), Some(Bopomofo::I), None }, { Some(Bopomofo::C), None, None }),
    amb!("si", { Some(Bopomofo::X), Some(Bopomofo::I), None }, { Some(Bopomofo::S), None, None }),
];

const MPS2_PINYIN_MAPPING: [AmbiguousMapEntry; 13] = [
    amb!("chi", { Some(Bopomofo::Q), Some(Bopomofo::I), None }, { Some(Bopomofo::CH), None, None }),
    amb!("shi", { Some(Bopomofo::X), Some(Bopomofo::I), None }, { Some(Bopomofo::SH), None, None }),
    amb!("ci", { Some(Bopomofo::Q), Some(Bopomofo::I), None }, { Some(Bopomofo::C), None, None }),
    amb!("si", { Some(Bopomofo::X), Some(Bopomofo::I), None }, { Some(Bopomofo::S), None, None }),
    amb!("niu", { Some(Bopomofo::N), Some(Bopomofo::IU), None }, { Some(Bopomofo::N), Some(Bopomofo::I), Some(Bopomofo::OU) }),
    amb!("liu", { Some(Bopomofo::L), Some(Bopomofo::IU), None }, { Some(Bopomofo::L), Some(Bopomofo::I), Some(Bopomofo::OU) }),
    amb!("jiu", { Some(Bopomofo::J), Some(Bopomofo::IU), None }, { Some(Bopomofo::J), Some(Bopomofo::I), Some(Bopomofo::OU) }),
    amb!("chiu", { Some(Bopomofo::Q), Some(Bopomofo::IU), None }, { Some(Bopomofo::Q), Some(Bopomofo::I), Some(Bopomofo::OU) }),
    amb!("shiu", { Some(Bopomofo::X), Some(Bopomofo::IU), None }, { Some(Bopomofo::X), Some(Bopomofo::I), Some(Bopomofo::OU) }),
    amb!("ju", { Some(Bopomofo::ZH), Some(Bopomofo::U), None }, { Some(Bopomofo::J), Some(Bopomofo::IU), None }),
    amb!("juan", { Some(Bopomofo::ZH), Some(Bopomofo::U), Some(Bopomofo::AN) }, { Some(Bopomofo::J), Some(Bopomofo::IU), Some(Bopomofo::AN) }),
    amb!("juen", { Some(Bopomofo::ZH), Some(Bopomofo::U), Some(Bopomofo::EN) }, { Some(Bopomofo::J), Some(Bopomofo::IU), Some(Bopomofo::EN) }),
    amb!("tzu", { Some(Bopomofo::Z), Some(Bopomofo::U), None }, { Some(Bopomofo::Z), None, None }),
];

const INITIAL_MAPPING: [InitialMapEntry; 25] = [
    ini!("tz", Bopomofo::Z),
    ini!("b", Bopomofo::B),
    ini!("p", Bopomofo::P),
    ini!("m", Bopomofo::M),
    ini!("f", Bopomofo::F),
    ini!("d", Bopomofo::D),
    ini!("ts", Bopomofo::C),
    ini!("t", Bopomofo::T),
    ini!("n", Bopomofo::N),
    ini!("l", Bopomofo::L),
    ini!("g", Bopomofo::G),
    ini!("k", Bopomofo::K),
    ini!("hs", Bopomofo::X),
    ini!("h", Bopomofo::H),
    ini!("jh", Bopomofo::ZH),
    ini!("j", Bopomofo::J),
    ini!("q", Bopomofo::Q),
    ini!("x", Bopomofo::X),
    ini!("zh", Bopomofo::ZH),
    ini!("ch", Bopomofo::CH),
    ini!("sh", Bopomofo::SH),
    ini!("r", Bopomofo::R),
    ini!("z", Bopomofo::Z),
    ini!("c", Bopomofo::C),
    ini!("s", Bopomofo::S),
];

const FINAL_MAPPING: [FinalMapEntry; 90] = [
    fin!("uang", Some(Bopomofo::U), Some(Bopomofo::ANG)),
    fin!("wang", Some(Bopomofo::U), Some(Bopomofo::ANG)),
    fin!("weng", Some(Bopomofo::U), Some(Bopomofo::ENG)),
    fin!("wong", Some(Bopomofo::U), Some(Bopomofo::ENG)),
    fin!("ying", Some(Bopomofo::I), Some(Bopomofo::ENG)),
    fin!("yung", Some(Bopomofo::IU), Some(Bopomofo::ENG)),
    fin!("yong", Some(Bopomofo::IU), Some(Bopomofo::ENG)),
    fin!("iung", Some(Bopomofo::IU), Some(Bopomofo::ENG)),
    fin!("iong", Some(Bopomofo::IU), Some(Bopomofo::ENG)),
    fin!("iang", Some(Bopomofo::I), Some(Bopomofo::ANG)),
    fin!("yang", Some(Bopomofo::I), Some(Bopomofo::ANG)),
    fin!("yuan", Some(Bopomofo::IU), Some(Bopomofo::AN)),
    fin!("iuan", Some(Bopomofo::IU), Some(Bopomofo::AN)),
    fin!("ing", Some(Bopomofo::I), Some(Bopomofo::ENG)),
    fin!("iao", Some(Bopomofo::I), Some(Bopomofo::AU)),
    fin!("iau", Some(Bopomofo::I), Some(Bopomofo::AU)),
    fin!("yao", Some(Bopomofo::I), Some(Bopomofo::AU)),
    fin!("yau", Some(Bopomofo::I), Some(Bopomofo::AU)),
    fin!("yun", Some(Bopomofo::IU), Some(Bopomofo::EN)),
    fin!("iun", Some(Bopomofo::IU), Some(Bopomofo::EN)),
    fin!("vn", Some(Bopomofo::IU), Some(Bopomofo::EN)),
    fin!("iou", Some(Bopomofo::I), Some(Bopomofo::OU)),
    fin!("iu", Some(Bopomofo::I), Some(Bopomofo::OU)),
    fin!("you", Some(Bopomofo::I), Some(Bopomofo::OU)),
    fin!("io", Some(Bopomofo::I), Some(Bopomofo::O)),
    fin!("yo", Some(Bopomofo::I), Some(Bopomofo::O)),
    fin!("ian", Some(Bopomofo::I), Some(Bopomofo::AN)),
    fin!("ien", Some(Bopomofo::I), Some(Bopomofo::AN)),
    fin!("yan", Some(Bopomofo::I), Some(Bopomofo::AN)),
    fin!("yen", Some(Bopomofo::I), Some(Bopomofo::AN)),
    fin!("yin", Some(Bopomofo::I), Some(Bopomofo::EN)),
    fin!("ang", None, Some(Bopomofo::ANG)),
    fin!("eng", None, Some(Bopomofo::ENG)),
    fin!("uei", Some(Bopomofo::U), Some(Bopomofo::EI)),
    fin!("ui", Some(Bopomofo::U), Some(Bopomofo::EI)),
    fin!("wei", Some(Bopomofo::U), Some(Bopomofo::EI)),
    fin!("uen", Some(Bopomofo::U), Some(Bopomofo::EN)),
    fin!("yueh", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("yue", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("iue", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("ueh", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("ue", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("ve", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("uai", Some(Bopomofo::U), Some(Bopomofo::AI)),
    fin!("wai", Some(Bopomofo::U), Some(Bopomofo::AI)),
    fin!("uan", Some(Bopomofo::U), Some(Bopomofo::AN)),
    fin!("wan", Some(Bopomofo::U), Some(Bopomofo::AN)),
    fin!("un", Some(Bopomofo::U), Some(Bopomofo::EN)),
    fin!("wen", Some(Bopomofo::U), Some(Bopomofo::EN)),
    fin!("wun", Some(Bopomofo::U), Some(Bopomofo::EN)),
    fin!("ung", Some(Bopomofo::U), Some(Bopomofo::ENG)),
    fin!("ong", Some(Bopomofo::U), Some(Bopomofo::ENG)),
    fin!("van", Some(Bopomofo::IU), Some(Bopomofo::AN)),
    fin!("er", None, Some(Bopomofo::ER)),
    fin!("ai", None, Some(Bopomofo::AI)),
    fin!("ei", None, Some(Bopomofo::EI)),
    fin!("ao", None, Some(Bopomofo::AU)),
    fin!("au", None, Some(Bopomofo::AU)),
    fin!("ou", None, Some(Bopomofo::OU)),
    fin!("an", None, Some(Bopomofo::AN)),
    fin!("en", None, Some(Bopomofo::EN)),
    fin!("yi", None, Some(Bopomofo::I)),
    fin!("ia", Some(Bopomofo::I), Some(Bopomofo::A)),
    fin!("ya", Some(Bopomofo::I), Some(Bopomofo::A)),
    fin!("ieh", Some(Bopomofo::I), Some(Bopomofo::EH)),
    fin!("ie", Some(Bopomofo::I), Some(Bopomofo::EH)),
    fin!("yeh", Some(Bopomofo::I), Some(Bopomofo::EH)),
    fin!("ye", Some(Bopomofo::I), Some(Bopomofo::EH)),
    fin!("in", Some(Bopomofo::I), Some(Bopomofo::EN)),
    fin!("wu", Some(Bopomofo::U), None),
    fin!("ua", Some(Bopomofo::U), Some(Bopomofo::A)),
    fin!("wa", Some(Bopomofo::U), Some(Bopomofo::A)),
    fin!("uo", Some(Bopomofo::U), Some(Bopomofo::O)),
    fin!("wo", Some(Bopomofo::U), Some(Bopomofo::O)),
    fin!("yu", Some(Bopomofo::IU), None),
    fin!("ve", Some(Bopomofo::IU), Some(Bopomofo::EH)),
    fin!("vn", Some(Bopomofo::IU), Some(Bopomofo::EN)),
    fin!("ih", None, None),
    fin!("a", None, Some(Bopomofo::A)),
    fin!("o", None, Some(Bopomofo::O)),
    fin!("eh", None, Some(Bopomofo::EH)),
    fin!("e", None, Some(Bopomofo::E)),
    fin!("v", Some(Bopomofo::IU), None),
    fin!("i", Some(Bopomofo::I), None),
    fin!("u", Some(Bopomofo::U), None),
    fin!("E", None, Some(Bopomofo::EH)),
    fin!("n", None, Some(Bopomofo::EN)),
    fin!("ng", None, Some(Bopomofo::ENG)),
    fin!("r", None, None),
    fin!("z", None, None),
];
