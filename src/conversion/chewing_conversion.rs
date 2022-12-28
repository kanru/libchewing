use std::{collections::HashMap, fmt::Debug};

use crate::{
    dictionary::{Dictionary, Phrase},
    zhuyin::Syllable,
};

use super::{Break, ChineseSequence, ConversionEngine, Interval};

#[derive(Debug)]
pub struct ChewingConversionEngine<D>
where
    D: Dictionary,
{
    dict: D,
}

impl<D> ChewingConversionEngine<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> ChewingConversionEngine<D> {
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
        'next_phrase: for phrase in self.dict.lookup_phrase(syllables) {
            // If there exists a user selected interval which is a
            // sub-interval of this phrase but the substring is
            // different then we can skip this phrase.
            for selection in selections.iter() {
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
            if phrase.freq() > max_freq {
                max_freq = phrase.freq();
                best_phrase = Some(phrase);
            }
        }

        best_phrase
    }

    fn calculate_score(&self, source_node: &Node, start: usize, end: usize, freq: usize) -> usize {
        let len = end - start;
        let reduction_factor = if len == 1 { 512 } else { 1 };
        // Heuristic: multiply frequency with length to boost the score of long phrases
        source_node.best_score + len * (freq / reduction_factor).max(1)
    }
}

impl<D> ConversionEngine for ChewingConversionEngine<D>
where
    D: Dictionary,
{
    fn convert(&self, sequence: &ChineseSequence) -> Vec<Interval> {
        // Calculate the best path. The graph is a DAG so we can scan from left
        // to right to find the longest-paths.
        let mut cache = HashMap::new();
        let mut dp = vec![Node::default(); sequence.syllables.len() + 1];
        for t in 0..=sequence.syllables.len() {
            for s in 0..t {
                let entry = cache.entry((s, t));
                if let Some(phrase) = entry.or_insert_with(|| {
                    self.find_best_phrase(
                        s,
                        &sequence.syllables[s..t],
                        &sequence.selections,
                        &sequence.breaks,
                    )
                }) {
                    let freq = phrase.freq();
                    let score = self.calculate_score(&dp[s], s, t, freq as usize);
                    if dp[t].best_score < score {
                        dp[t] = Node {
                            best_source: s,
                            best_score: score,
                            best_phrase: Some(phrase.clone()),
                        };
                    }
                };
            }
        }

        let mut result = vec![];
        let mut end = sequence.syllables.len();
        let mut start = dp[end].best_source;
        loop {
            let phrase = dp[end]
                .best_phrase
                .as_ref()
                .expect("all solutions should have valid phrases");
            result.push(Interval {
                start,
                end,
                phrase: phrase.to_string(),
            });
            if start == 0 {
                break;
            }
            end = start;
            start = dp[end].best_source;
        }
        result
    }

    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval> {
        // Use Yen's algorithm to find the Kth solution
        //
        // Ref: https://en.m.wikipedia.org/wiki/Yen%27s_algorithm
        self.convert(segment)
    }
}

#[derive(Clone, Default, Debug)]
struct Node {
    best_source: usize,
    best_score: usize,
    best_phrase: Option<Phrase>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        conversion::{Break, ChineseSequence, ConversionEngine, Interval},
        dictionary::Phrase,
        syl,
        zhuyin::{Bopomofo::*, Syllable},
    };

    use super::ChewingConversionEngine;

    fn test_dictionary() -> HashMap<Vec<Syllable>, Vec<Phrase>> {
        HashMap::from([
            (vec![syl![G, U, O, TONE2]], vec![("國", 1).into()]),
            (vec![syl![M, I, EN, TONE4]], vec![("民", 1).into()]),
            (vec![syl![D, A, TONE4]], vec![("大", 1).into()]),
            (vec![syl![H, U, EI, TONE4]], vec![("會", 1).into()]),
            (vec![syl![D, AI, TONE4]], vec![("代", 1).into()]),
            (vec![syl![B, I, AU, TONE3]], vec![("表", 1).into()]),
            (
                vec![syl![G, U, O, TONE2], syl![M, I, EN, TONE4]],
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
        ])
    }

    #[test]
    fn convert_simple_chinese_sequence() {
        let dict = test_dictionary();
        let engine = ChewingConversionEngine::new(dict);
        let sequence = ChineseSequence {
            syllables: vec![
                syl![G, U, O, TONE2],
                syl![M, I, EN, TONE4],
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
                    start: 4,
                    end: 6,
                    phrase: "代表".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
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
                syl![M, I, EN, TONE4],
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
                    start: 5,
                    end: 6,
                    phrase: "表".to_string()
                },
                Interval {
                    start: 4,
                    end: 5,
                    phrase: "代".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 1,
                    end: 2,
                    phrase: "民".to_string()
                },
                Interval {
                    start: 0,
                    end: 1,
                    phrase: "國".to_string()
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
                syl![M, I, EN, TONE4],
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
                    start: 4,
                    end: 6,
                    phrase: "戴錶".to_string()
                },
                Interval {
                    start: 2,
                    end: 4,
                    phrase: "大會".to_string()
                },
                Interval {
                    start: 0,
                    end: 2,
                    phrase: "國民".to_string()
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
}
