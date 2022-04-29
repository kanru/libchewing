//! Keyboard layout conversion to phonetic keys
//!
//! This module contains engines for phonetic key conversions.
//!
//! Traditionally the keyboards sold in Chinese speaking region have
//! both the English alphabets and Zhuyin symbols printed on the keys.
//! Like English keyboards can have different layouts (QWERTY, DVORAK, etc.),
//! Zhuyin keyboards also have different layouts.
//!
//! The most widely used Zhuyin layout is the one directly printed on the keyboards.
//! It is a one to one mapping from keys to Zhuyin symbols. However, some layouts
//! have smarter mapping from keys to Zhuyin symbols, taking advantage of impossible
//! combinations, to reduce the total keys required.
//!
//! Chewing currently supports the default layout, Hsu's layout, ET26 layout,
//! DaChen CP26 layout, and the Pinyin layout.
//!
//! Since people usually practice Zhuyin input method independently from practicing
//! English typing, they acquire different muscle memory. This module provides APIs
//! to map different English layouts to layout independent key indexes that can be
//! used to drive the layout engines.

pub mod hsu;

enum KeyBehavior {
    Ignore = 0,
    Absorb,
    Commit,
    KeyError,
    Error,
    NoWord,
    OpenSymbolTable,
}

/// Layout independent key index
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub enum KeyIndex {
    K0,
//  1   2   3   4   5   6   7   8   9   0    -    =    \    `
    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14,
//    Q    W    E    R    T    Y    U    I    O    P    [    ]
      K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26,
//      A    S    D    F    G    H    J    K    L    ;   '
        K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K37,
//        Z    X    C    V    B    N    M    ,    .    /    SPC
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48
}

/// USB HID KeyCodes
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub enum KeyCode {
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
      Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
       A, S, D, F, G, H, J, K, L, SColon, Quote,
        Z, X, C, V, B, N, M, Comma, Dot, Slash, Space
}

macro_rules! match_index {
    ($target:expr,
     $k1:literal,
     $k2:literal,
     $k3:literal,
     $k4:literal,
     $k5:literal,
     $k6:literal,
     $k7:literal,
     $k8:literal,
     $k9:literal,
     $k10:literal,
     $k11:literal,
     $k12:literal,
     $k13:literal,
     $k14:literal,
     $k15:literal,
     $k16:literal,
     $k17:literal,
     $k18:literal,
     $k19:literal,
     $k20:literal,
     $k21:literal,
     $k22:literal,
     $k23:literal,
     $k24:literal,
     $k25:literal,
     $k26:literal,
     $k27:literal,
     $k28:literal,
     $k29:literal,
     $k30:literal,
     $k31:literal,
     $k32:literal,
     $k33:literal,
     $k34:literal,
     $k35:literal,
     $k36:literal,
     $k37:literal,
     $k38:literal,
     $k39:literal,
     $k40:literal,
     $k41:literal,
     $k42:literal,
     $k43:literal,
     $k44:literal,
     $k45:literal,
     $k46:literal,
     $k47:literal,
     $k48:literal
    ) => {
        match $target {
            $k1 => K1,
            $k2 => K2,
            $k3 => K3,
            $k4 => K4,
            $k5 => K5,
            $k6 => K6,
            $k7 => K7,
            $k8 => K8,
            $k9 => K9,
            $k10 => K10,
            $k11 => K11,
            $k12 => K12,
            $k13 => K13,
            $k14 => K14,
            $k15 => K15,
            $k16 => K16,
            $k17 => K17,
            $k18 => K18,
            $k19 => K19,
            $k20 => K20,
            $k21 => K21,
            $k22 => K22,
            $k23 => K23,
            $k24 => K24,
            $k25 => K25,
            $k26 => K26,
            $k27 => K27,
            $k28 => K28,
            $k29 => K29,
            $k30 => K30,
            $k31 => K31,
            $k32 => K32,
            $k33 => K33,
            $k34 => K34,
            $k35 => K35,
            $k36 => K36,
            $k37 => K37,
            $k38 => K38,
            $k39 => K39,
            $k40 => K40,
            $k41 => K41,
            $k42 => K42,
            $k43 => K43,
            $k44 => K44,
            $k45 => K45,
            $k46 => K46,
            $k47 => K47,
            $k48 => K48,
            _ => K0,
        }
    };
}

pub trait KeyIndexFromQwerty {
    fn as_key_index(&self) -> KeyIndex;
}

#[rustfmt::skip]
impl KeyIndexFromQwerty for u8 {
    fn as_key_index(&self) -> KeyIndex {
        match_index!(self,
            b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\\', b'`',
             b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']',
              b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'',
               b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', b' '
        )
    }
}

use KeyCode::*;
use KeyIndex::*;

#[rustfmt::skip]
pub const BLANK: [KeyIndex; 48] = [
    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14,
      K15, K16, K17, K18, K19, K20, K21, K22, K23, K24, K25, K26,
        K27, K28, K29, K30, K31, K32, K33, K34, K35, K36, K37,
          K38, K39, K40, K41, K42, K43, K44, K45, K46, K47, K48
];

#[rustfmt::skip]
pub const QWERTY: [KeyCode; 48] = [
    N1, N2, N3, N4, N5, N6, N7, N8, N9, N0, Minus, Equal, BSlash, Grave,
      Q, W, E, R, T, Y, U, I, O, P, LBracket, RBracket,
       A, S, D, F, G, H, J, K, L, SColon, Quote,
        Z, X, C, V, B, N, M, Comma, Dot, Slash, Space
];
