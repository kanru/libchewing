use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

use crate::{
    dictionary::{Dictionary, Phrase},
    zhuyin::Syllable,
};

use super::{Break, ChineseSequence, ConversionEngine, Interval};

#[derive(Debug)]
pub struct ExperimentalConversionEngine {
    dict: Rc<RefCell<dyn Dictionary>>,
}

impl ExperimentalConversionEngine {
    pub fn new(dict: Rc<RefCell<dyn Dictionary>>) -> ExperimentalConversionEngine {
        ExperimentalConversionEngine { dict }
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

    fn find_best_path(
        &self,
        graph: &mut Graph,
        sequence: &ChineseSequence,
        source: usize,
        target: usize,
    ) -> Path {
        debug_assert!(target <= sequence.syllables.len());

        // Calculate the best path. The graph is a DAG so we can scan from left
        // to right to find the longest-paths.
        let mut dp = vec![Node::default(); target + 1];
        for t in source..=target {
            for s in source..t {
                if !graph.is_edge_possible(s, t) {
                    continue;
                }
                let entry = graph.entry(s, t);
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

        Path::from_dp(dp, target)
    }

    fn find_all_paths(
        &self,
        graph: &mut Graph,
        sequence: &ChineseSequence,
        source: usize,
        target: usize,
        prefix: Option<Path>,
    ) -> Vec<Path> {
        if source == target {
            return vec![prefix.expect("should have prefix")];
        }
        let mut result = vec![];
        for t in source..=target {
            let entry = graph.entry(source, t);
            if let Some(phrase) = entry.or_insert_with(|| {
                self.find_best_phrase(
                    source,
                    &sequence.syllables[source..t],
                    &sequence.selections,
                    &sequence.breaks,
                )
            }) {
                let mut prefix = prefix.clone().unwrap_or_default();
                prefix.score += 1;
                prefix.intervals.push(Interval {
                    start: source,
                    end: t,
                    phrase: phrase.to_string(),
                });
                result.append(&mut self.find_all_paths(graph, sequence, t, target, Some(prefix)));
            }
        }
        result
    }
}

impl ConversionEngine for ExperimentalConversionEngine {
    fn convert(&self, sequence: &ChineseSequence) -> Vec<Interval> {
        if sequence.syllables.is_empty() {
            return vec![];
        }
        let mut graph = Graph::default();
        self.find_best_path(&mut graph, sequence, 0, sequence.syllables.len())
            .intervals
    }

    fn convert_next(&self, sequence: &ChineseSequence, next: usize) -> Vec<Interval> {
        // TODO: Use modified Yen's algorithm to find the Kth solution
        if sequence.syllables.is_empty() {
            return vec![];
        }
        let mut graph = Graph::default();
        let mut paths =
            self.find_all_paths(&mut graph, sequence, 0, sequence.syllables.len(), None);
        paths.sort();
        paths
            .into_iter()
            .cycle()
            .skip(next)
            .next()
            .map(|p| p.intervals)
            .expect("should have path")
    }
}

#[derive(Default)]
struct Graph {
    edges_score: HashMap<(usize, usize), Option<Phrase>>,
    removed_edges: HashSet<(usize, usize)>,
    removed_nodes: HashSet<usize>,
}

impl Graph {
    fn remove_edge(&mut self, s: usize, t: usize) {
        self.removed_edges.insert((s, t));
    }
    fn remove_node(&mut self, node: usize) {
        self.removed_nodes.insert(node);
    }
    fn restore_removed(&mut self) {
        self.removed_edges.clear();
        self.removed_nodes.clear();
    }
    fn is_edge_possible(&self, s: usize, t: usize) -> bool {
        !self.removed_nodes.contains(&s)
            && !self.removed_nodes.contains(&t)
            && !self.removed_edges.contains(&(s, t))
    }
    fn entry(&mut self, s: usize, t: usize) -> Entry<(usize, usize), Option<Phrase>> {
        debug_assert!(
            self.is_edge_possible(s, t),
            "edge {:?} is removed from the graph",
            (s, t)
        );
        self.edges_score.entry((s, t))
    }
}

#[derive(Clone, Default, Debug)]
struct Node {
    best_source: usize,
    best_score: usize,
    best_phrase: Option<Phrase>,
}

#[derive(Default, Clone)]
struct Path {
    score: usize,
    intervals: Vec<Interval>,
}

impl Path {
    fn from_dp(dp: Vec<Node>, end: usize) -> Path {
        let mut intervals = vec![];
        let mut end = end;
        let mut start = dp[end].best_source;
        let mut score = 0;
        loop {
            let phrase = dp[end]
                .best_phrase
                .as_ref()
                .expect("all solutions should have valid phrases");
            score += dp[end].best_score;
            intervals.push(Interval {
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
        Path { score, intervals }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Path {}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::{
        conversion::{Break, ChineseSequence, ConversionEngine, Interval},
        dictionary::{Dictionary, Phrase},
        syl,
        zhuyin::{Bopomofo::*, Syllable},
    };

    use super::ExperimentalConversionEngine;

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
        let engine = ExperimentalConversionEngine::new(dict);
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
        let engine = ExperimentalConversionEngine::new(dict);
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
        let engine = ExperimentalConversionEngine::new(dict);
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
        let engine = ExperimentalConversionEngine::new(dict);
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
        let engine = ExperimentalConversionEngine::new(dict);
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
        let engine = ExperimentalConversionEngine::new(dict);
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
