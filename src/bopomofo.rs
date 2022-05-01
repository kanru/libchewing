use thiserror::Error;

/// The category of the phonetic symbols
///
/// Zhuyin, or Bopomofo, consists of 37 letters and 4 tone marks. They are
/// categorized into one of the four categories:
///
/// 1. Initial sounds: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
/// 2. Medial glides: ㄧㄨㄩ
/// 3. Final rhymes: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
/// 4. Tonal marks: ˙ˊˇˋ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BopomofoKind {
    Initial = 0,
    MedialGlide,
    Final,
    Tone,
}

/// Zhuyin Fuhao, often shortened as zhuyin and commonly called bopomofo
///
/// <https://simple.m.wikipedia.org/wiki/Zhuyin>
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bopomofo {
    /// ㄅ
    B = 0,
    /// ㄆ
    P,
    /// ㄇ
    M,
    /// ㄈ
    F,
    /// ㄉ
    D,
    /// ㄊ
    T,
    /// ㄋ
    N,
    /// ㄌ
    L,
    /// ㄍ
    G,
    /// ㄎ
    K,
    /// ㄏ
    H,
    /// ㄐ
    J,
    /// ㄑ
    Q,
    /// ㄒ
    X,
    /// ㄓ
    ZH,
    /// ㄔ
    CH,
    /// ㄕ
    SH,
    /// ㄖ
    R,
    /// ㄗ
    Z,
    /// ㄘ
    C,
    /// 一
    I,
    /// ㄨ
    U,
    /// ㄩ
    IU,
    /// ㄙ
    S,
    /// ㄚ
    A,
    /// ㄛ
    O,
    /// ㄜ
    E,
    /// ㄝ
    EH,
    /// ㄞ
    AI,
    /// ㄟ
    EI,
    /// ㄠ
    AU,
    /// ㄡ
    OU,
    /// ㄢ
    AN,
    /// ㄣ
    EN,
    /// ㄤ
    ANG,
    /// ㄥ
    ENG,
    /// ㄦ
    ER,
    /// ˉ
    TONE1,
    /// ˊ
    TONE2,
    /// ˇ
    TONE3,
    /// ˋ
    TONE4,
    /// ˙
    TONE5,
}

use Bopomofo::*;

const INITIAL_MAP: [Bopomofo; 21] = [
    B, P, M, F, D, T, N, L, G, K, H, J, Q, X, ZH, CH, SH, R, Z, C, S,
];
const MEDIAL_MAP: [Bopomofo; 3] = [I, U, IU];
const FINAL_MAP: [Bopomofo; 13] = [A, O, E, EH, AI, EI, AU, OU, AN, EN, ANG, ENG, ER];
const TONE_MAP: [Bopomofo; 4] = [TONE5, TONE2, TONE3, TONE4];

impl Bopomofo {
    pub fn kind(&self) -> BopomofoKind {
        match self {
            B | P | M | F | D | T | N | L | G | K | H | J | Q | X | ZH | CH | SH | R | Z | C
            | S => BopomofoKind::Initial,
            I | U | IU => BopomofoKind::MedialGlide,
            A | O | E | EH | AI | EI | AU | OU | AN | EN | ANG | ENG | ER => BopomofoKind::Final,
            TONE1 | TONE2 | TONE3 | TONE4 | TONE5 => BopomofoKind::Tone,
        }
    }
    pub fn from_initial(index: i32) -> Bopomofo {
        INITIAL_MAP[(index - 1) as usize]
    }
    pub fn from_medial(index: i32) -> Bopomofo {
        MEDIAL_MAP[(index - 1) as usize]
    }
    pub fn from_final(index: i32) -> Bopomofo {
        FINAL_MAP[(index - 1) as usize]
    }
    pub fn from_tone(index: i32) -> Bopomofo {
        TONE_MAP[(index - 1) as usize]
    }

    pub fn initial_index(&self) -> i32 {
        (INITIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as i32
    }
    pub fn medial_index(&self) -> i32 {
        (MEDIAL_MAP.iter().position(|b| b == self).unwrap() + 1) as i32
    }
    pub fn final_index(&self) -> i32 {
        (FINAL_MAP.iter().position(|b| b == self).unwrap() + 1) as i32
    }
    pub fn tone_index(&self) -> i32 {
        (TONE_MAP.iter().position(|b| b == self).unwrap() + 1) as i32
    }
}

#[derive(Error, Debug)]
pub enum BopomofoParseError {
    #[error("unknown symbol")]
    Unknown,
}

impl TryFrom<char> for Bopomofo {
    type Error = BopomofoParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'ㄅ' => Ok(Bopomofo::B),
            'ㄆ' => Ok(Bopomofo::P),
            'ㄇ' => Ok(Bopomofo::M),
            'ㄈ' => Ok(Bopomofo::F),
            'ㄉ' => Ok(Bopomofo::D),
            'ㄊ' => Ok(Bopomofo::T),
            'ㄋ' => Ok(Bopomofo::N),
            'ㄌ' => Ok(Bopomofo::L),
            'ㄍ' => Ok(Bopomofo::G),
            'ㄎ' => Ok(Bopomofo::K),
            'ㄏ' => Ok(Bopomofo::H),
            'ㄐ' => Ok(Bopomofo::J),
            'ㄑ' => Ok(Bopomofo::Q),
            'ㄒ' => Ok(Bopomofo::X),
            'ㄓ' => Ok(Bopomofo::ZH),
            'ㄔ' => Ok(Bopomofo::CH),
            'ㄕ' => Ok(Bopomofo::SH),
            'ㄖ' => Ok(Bopomofo::R),
            'ㄗ' => Ok(Bopomofo::Z),
            'ㄘ' => Ok(Bopomofo::C),
            'ㄙ' => Ok(Bopomofo::S),
            'ㄚ' => Ok(Bopomofo::A),
            'ㄛ' => Ok(Bopomofo::O),
            'ㄜ' => Ok(Bopomofo::E),
            'ㄝ' => Ok(Bopomofo::EH),
            'ㄞ' => Ok(Bopomofo::AI),
            'ㄟ' => Ok(Bopomofo::EI),
            'ㄠ' => Ok(Bopomofo::AU),
            'ㄡ' => Ok(Bopomofo::OU),
            'ㄢ' => Ok(Bopomofo::AN),
            'ㄣ' => Ok(Bopomofo::EN),
            'ㄤ' => Ok(Bopomofo::ANG),
            'ㄥ' => Ok(Bopomofo::ENG),
            'ㄦ' => Ok(Bopomofo::ER),
            'ㄧ' => Ok(Bopomofo::I),
            'ㄨ' => Ok(Bopomofo::U),
            'ㄩ' => Ok(Bopomofo::IU),
            'ˉ' => Ok(Bopomofo::TONE1),
            'ˊ' => Ok(Bopomofo::TONE2),
            'ˇ' => Ok(Bopomofo::TONE3),
            'ˋ' => Ok(Bopomofo::TONE4),
            '˙' => Ok(Bopomofo::TONE5),
            _ => Err(BopomofoParseError::Unknown),
        }
    }
}
