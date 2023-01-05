use std::{cell::RefCell, ops::Neg, rc::Rc};

use crate::{
    dictionary::{Dictionary, Phrase},
    zhuyin::Syllable,
};

use super::{Break, ChineseSequence, ConversionEngine, Interval};

#[derive(Debug)]
pub struct ChewingConversionEngine {
    dict: Rc<RefCell<dyn Dictionary>>,
}

impl ChewingConversionEngine {
    pub fn new(dict: Rc<RefCell<dyn Dictionary>>) -> ChewingConversionEngine {
        ChewingConversionEngine { dict }
    }

    fn find_best_phrase(
        &self,
        offset: usize,
        syllables: &[Syllable],
        selections: &[Interval],
        breaks: &[Break],
    ) -> Option<Phrase> {
        let start = offset;
        let end = offset + syllables.len();

        for br in breaks.iter() {
            if br.0 > start && br.0 < end {
                // There exists a break point that forbids connecting these
                // syllables.
                return None;
            }
        }

        let mut max_freq = 0;
        let mut best_phrase = None;
        'next_phrase: for phrase in self.dict.borrow().lookup_phrase(syllables) {
            // If there exists a user selected interval which is a
            // sub-interval of this phrase but the substring is
            // different then we can skip this phrase.
            for selection in selections.iter() {
                debug_assert!(!selection.phrase.is_empty());
                if start <= selection.start && end >= selection.end {
                    let offset = selection.start - start;
                    let len = selection.end - selection.start;
                    let substring: String =
                        phrase.as_str().chars().skip(offset).take(len).collect();
                    if substring != selection.phrase {
                        continue 'next_phrase;
                    }
                }
            }

            // If there are phrases that can satisfy all the constraints
            // then pick the one with highest frequency.
            if best_phrase.is_none() || phrase.freq() > max_freq {
                max_freq = phrase.freq();
                best_phrase = Some(phrase);
            }
        }

        best_phrase
    }
    fn find_intervals(&self, seq: &ChineseSequence) -> Vec<PossibleInterval> {
        let mut intervals = vec![];
        for begin in 0..seq.syllables.len() {
            for end in begin..=seq.syllables.len() {
                if let Some(phrase) = self.find_best_phrase(
                    begin,
                    &seq.syllables[begin..end],
                    &seq.selections,
                    &seq.breaks,
                ) {
                    intervals.push(PossibleInterval {
                        start: begin,
                        end,
                        phrase,
                    });
                }
            }
        }
        intervals
    }
    fn set_info(&self, seq: &ChineseSequence, intervals: &[PossibleInterval]) -> TreeData {
        let len = seq.syllables.len();
        let mut left_most = vec![0usize; len];
        let mut graph = vec![vec![false; len]; len];

        for i in 0..len {
            left_most[i] = i;
        }
        for i in 0..intervals.len() {
            graph[intervals[i].start][intervals[i].end] = true;
            graph[intervals[i].end][intervals[i].start] = true;
        }
        for j in 0..len {
            for i in 0..len {
                if !graph[j][i] {
                    continue;
                }
                if left_most[i] < left_most[j] {
                    left_most[j] = left_most[i];
                }
            }
        }
        TreeData { left_most, graph }
    }
    /// Remove the interval containing in another interval.
    ///
    /// Example:
    /// 國民大會 has three interval: 國民, 大會, 國民大會. This function removes
    /// 國民, 大會 because 國民大會 contains 國民 and 大會.
    fn discard1(&self, intervals: Vec<PossibleInterval>) -> Vec<PossibleInterval> {
        let untouched = intervals.clone();
        intervals
            .into_iter()
            .filter(|it| !untouched.iter().any(|u| u != it && u.contains(it)))
            .collect()
    }
    // Remove the interval that cannot connect to head or tail by other intervals.
    //
    // Example:
    // The input string length is 5
    // The available intervals are [1,1], [1,2], [2,3], [2,4], [5,5], [3,5].
    //
    // The possible connection from head to tail are [1,2][3,5], and
    // [1,1][2,4][5,5]. Since [2,3] cannot connect to head or tail, it is removed
    // by this function.
    fn discard2(&self, intervals: Vec<PossibleInterval>) -> Vec<PossibleInterval> {
        // XXX do we really need this optimization?
        intervals
    }
    fn dp_phrasing(&self, len: usize, mut intervals: Vec<PossibleInterval>) -> Vec<Interval> {
        // Assume P(x,y) is the highest score phrasing result from x to y. The
        // following is formula for P(x,y):
        //
        // P(x,y) = MAX( P(x,y-1)+P(y-1,y), P(x,y-2)+P(y-2,y), ... )
        //
        // While P(x,y-1) is stored in highest_score array, and P(y-1,y) is
        // interval end at y. In this formula, x is always 0.
        //
        // The format of highest_score array is described as following:
        //
        // highest_score[0] = P(0,0)
        // highest_score[1] = P(0,1)
        // ...
        // highest_score[y-1] = P(0,y-1)

        let mut highest_score = vec![RecordNode::default(); len + 1];

        // The interval shall be sorted by the increase order of end.
        intervals.sort_by(|a, b| a.end.cmp(&b.end));

        for i in 0..intervals.len() {
            let start = intervals[i].start;
            let end = intervals[i].end;

            let mut record = highest_score[start].clone();
            record.interval_index.push(i);

            record.score = 0;
            record.score += 1000 * self.rule_largest_sum(&record.interval_index, &intervals);
            record.score += 1000 * self.rule_largest_avgwordlen(&record.interval_index, &intervals);
            record.score +=
                100 * self.rule_smallest_lenvariance(&record.interval_index, &intervals);
            record.score += self.rule_largest_freqsum(&record.interval_index, &intervals);

            if highest_score[end].score < record.score {
                highest_score[end] = record;
            }
        }

        highest_score[len]
            .interval_index
            .iter()
            .map(|&i| intervals[i].clone().into())
            .collect()
    }

    fn rule_largest_sum(&self, interval_index: &[usize], intervals: &[PossibleInterval]) -> i32 {
        let mut score = 0;
        for &i in interval_index {
            score += intervals[i].end - intervals[i].start;
        }
        score as i32
    }

    fn rule_largest_avgwordlen(
        &self,
        interval_index: &[usize],
        intervals: &[PossibleInterval],
    ) -> i32 {
        // Constant factor 6=1*2*3, to keep value as integer
        6 * self.rule_largest_sum(interval_index, intervals)
            / i32::try_from(interval_index.len()).expect("number of intervals should be small")
    }

    fn rule_smallest_lenvariance(
        &self,
        interval_index: &[usize],
        intervals: &[PossibleInterval],
    ) -> i32 {
        let len = interval_index.len();
        let mut score = 0;
        // kcwu: heuristic? why variance no square function?
        for i in 0..len {
            for j in i + 1..len {
                let interval_1 = &intervals[interval_index[i]];
                let interval_2 = &intervals[interval_index[j]];
                score += interval_1.len().abs_diff(interval_2.len());
            }
        }
        i32::try_from(score).expect("score should fit in i32").neg()
    }

    fn rule_largest_freqsum(
        &self,
        interval_index: &[usize],
        intervals: &[PossibleInterval],
    ) -> i32 {
        let mut score = 0;
        for &i in interval_index {
            let interval = &intervals[i];
            let reduction_factor = if interval.len() == 1 { 512 } else { 1 };
            score += interval.phrase.freq() / reduction_factor;
        }
        i32::try_from(score).expect("score should fit in i32")
    }
}

struct TreeData {
    left_most: Vec<usize>,
    graph: Vec<Vec<bool>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PossibleInterval {
    start: usize,
    end: usize,
    phrase: Phrase,
}

impl PossibleInterval {
    pub fn contains(&self, other: &PossibleInterval) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl From<PossibleInterval> for Interval {
    fn from(value: PossibleInterval) -> Self {
        Interval {
            start: value.start,
            end: value.end,
            phrase: value.phrase.to_string(),
        }
    }
}

#[derive(Default, Clone, Debug)]
struct RecordNode {
    interval_index: Vec<usize>,
    score: i32,
    n_match_connect: usize,
    next: Option<Box<RecordNode>>,
}

impl ConversionEngine for ChewingConversionEngine {
    fn convert(&self, segment: &ChineseSequence) -> Vec<Interval> {
        self.convert_next(segment, 0)
    }

    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval> {
        let mut intervals = self.find_intervals(&segment);
        // intervals = self.discard1(intervals);
        // intervals = dbg!(self.discard2(intervals));
        // let mut tree_data = self.set_info(&segment, &intervals);

        self.dp_phrasing(segment.syllables.len(), intervals)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::{
        conversion::{Break, ChineseSequence, ConversionEngine, Interval},
        dictionary::Dictionary,
        syl,
        zhuyin::Bopomofo::*,
    };

    use super::ChewingConversionEngine;

    fn test_dictionary() -> Rc<RefCell<dyn Dictionary>> {
        Rc::new(RefCell::new(HashMap::from([
            (vec![syl![G, U, O, TONE2]], vec![("國", 1).into()]),
            (vec![syl![M, I, EN, TONE2]], vec![("民", 1).into()]),
            (vec![syl![D, A, TONE4]], vec![("大", 1).into()]),
            (vec![syl![H, U, EI, TONE4]], vec![("會", 1).into()]),
            (vec![syl![D, AI, TONE4]], vec![("代", 1).into()]),
            (vec![syl![B, I, AU, TONE3]], vec![("表", 1).into()]),
            (
                vec![syl![G, U, O, TONE2], syl![M, I, EN, TONE2]],
                vec![("國民", 200).into()],
            ),
            (
                vec![syl![D, A, TONE4], syl![H, U, EI, TONE4]],
                vec![("大會", 200).into()],
            ),
            (
                vec![syl![D, AI, TONE4], syl![B, I, AU, TONE3]],
                vec![("代表", 200).into(), ("戴錶", 100).into()],
            ),
            (vec![syl![X, I, EN]], vec![("心", 1).into()]),
            (
                vec![syl![K, U, TONE4], syl![I, EN]],
                vec![("庫音", 300).into()],
            ),
            (
                vec![syl![X, I, EN], syl![K, U, TONE4], syl![I, EN]],
                vec![("新酷音", 200).into()],
            ),
        ])))
    }

    #[test]
    fn convert_empty_sequence() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![],
            selections: vec![],
            breaks: vec![],
        };
        assert_eq!(Vec::<Interval>::new(), engine.convert(&sequence));
    }

    #[test]
    fn convert_simple_chinese_sequence() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![
                syl![G, U, O, TONE2],
                syl![M, I, EN, TONE2],
                syl![D, A, TONE4],
                syl![H, U, EI, TONE4],
                syl![D, AI, TONE4],
                syl![B, I, AU, TONE3],
            ],
            selections: vec![],
            breaks: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert(&sequence)
        );
    }

    #[test]
    fn convert_chinese_sequence_with_breaks() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![
                syl![G, U, O, TONE2],
                syl![M, I, EN, TONE2],
                syl![D, A, TONE4],
                syl![H, U, EI, TONE4],
                syl![D, AI, TONE4],
                syl![B, I, AU, TONE3],
            ],
            selections: vec![],
            breaks: vec![Break(1), Break(5)],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    phrase: "國".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    phrase: "民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 5,
                    phrase: "代".to_string()
                },
                Interval {
                    start: 5,
                    end: 6,
                    phrase: "表".to_string()
                },
            ],
            engine.convert(&sequence)
        );
    }

    #[test]
    fn convert_chinese_sequence_with_good_selection() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![
                syl![G, U, O, TONE2],
                syl![M, I, EN, TONE2],
                syl![D, A, TONE4],
                syl![H, U, EI, TONE4],
                syl![D, AI, TONE4],
                syl![B, I, AU, TONE3],
            ],
            selections: vec![Interval {
                start: 4,
                end: 6,
                phrase: "戴錶".to_string(),
            }],
            breaks: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "戴錶".to_string()
                },
            ],
            engine.convert(&sequence)
        );
    }

    #[test]
    fn convert_chinese_sequence_with_substring_selection() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![syl![X, I, EN], syl![K, U, TONE4], syl![I, EN]],
            selections: vec![Interval {
                start: 1,
                end: 3,
                phrase: "酷音".to_string(),
            }],
            breaks: vec![],
        };
        assert_eq!(
            vec![Interval {
                start: 0,
                end: 3,
                phrase: "新酷音".to_string()
            },],
            engine.convert(&sequence)
        );
    }

    #[test]
    fn convert_cycle_alternatives() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![
                syl![G, U, O, TONE2],
                syl![M, I, EN, TONE2],
                syl![D, A, TONE4],
                syl![H, U, EI, TONE4],
                syl![D, AI, TONE4],
                syl![B, I, AU, TONE3],
            ],
            selections: vec![],
            breaks: vec![],
        };
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert_next(&sequence, 0)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    phrase: "國".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    phrase: "民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert_next(&sequence, 1)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 3,
                    phrase: "大".to_string()
                },
                Interval {
                    start: 3,
                    end: 4,
                    phrase: "會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert_next(&sequence, 2)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 5,
                    phrase: "代".to_string()
                },
                Interval {
                    start: 5,
                    end: 6,
                    phrase: "表".to_string()
                }
            ],
            engine.convert_next(&sequence, 3)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 1,
                    phrase: "國".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    phrase: "民".to_string()
                },
                Interval {
                    start: 2,
                    end: 3,
                    phrase: "大".to_string()
                },
                Interval {
                    start: 3,
                    end: 4,
                    phrase: "會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                }
            ],
            engine.convert_next(&sequence, 4)
        );
        assert_eq!(
            vec![
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
            ],
            engine.convert_next(&sequence, 8)
        );
    }
}
