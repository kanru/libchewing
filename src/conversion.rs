use crate::zhuyin::Syllable;

pub use chewing_conversion::ChewingConversionEngine;

mod chewing_conversion;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval {
    start: usize,
    end: usize,
    phrase: String,
}

pub struct Break(usize);

// impl Interval {
//     pub fn contains(&self, other: &Interval) -> bool {
//         self.start <= other.start && self.end >= other.end
//     }
//     pub fn intersects(&self, other: &Interval) -> bool {
//         self.start.max(other.start) < self.end.min(other.end)
//     }
// }

pub struct ChineseSequence {
    syllables: Vec<Syllable>,
    selections: Vec<Interval>,
    breaks: Vec<Break>,
}

pub trait ConversionEngine {
    fn convert(&self, segment: &ChineseSequence) -> Vec<Interval>;
    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval>;
}
