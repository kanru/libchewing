use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::mem;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct Node {
    next: BTreeMap<Bopomofo, Box<Node>>,
    stem: Vec<Bopomofo>,
    phrases: BTreeSet<String>,
}

impl Node {
    pub fn find(&self, bopomofo: Bopomofo) -> Option<&Box<Node>> {
        self.next.get(&bopomofo)
    }

    pub fn find_mut(&mut self, bopomofo: Bopomofo) -> Option<&mut Box<Node>> {
        self.next.get_mut(&bopomofo)
    }

    pub fn link(&mut self, bopomofo: Bopomofo, node: Box<Node>) {
        self.next.insert(bopomofo, node);
    }

    pub fn add_phrase(&mut self, phrase: String) {
        self.phrases.insert(phrase);
    }

    pub fn phrases(&self) -> impl Iterator<Item = &String> {
        self.phrases.iter()
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            next: BTreeMap::new(),
            stem: Vec::new(),
            phrases: BTreeSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Dictionary {
    root: Node,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Bopomofo {
    B,
    P,
    M,
    F,
    D,
    T,
    N,
    L,
    G,
    K,
    H,
    J,
    Q,
    X,
    ZH,
    CH,
    SH,
    R,
    Z,
    C,
    S,
    A,
    O,
    E,
    EH,
    AI,
    EI,
    AU,
    OU,
    AN,
    EN,
    ANG,
    ENG,
    ER,
    I,
    U,
    IU,
    TONE1,
    TONE2,
    TONE3,
    TONE4,
    TONE5,
}

#[derive(Error, Debug)]
pub enum BopomofoParseError {
    #[error("unknown symbol")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum DictionaryError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("invalid input")]
    Invalid(#[from] BopomofoParseError),
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

impl Dictionary {
    fn new() -> Dictionary {
        Dictionary {
            root: Node::default(),
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Dictionary, DictionaryError> {
        let mut dict = Dictionary::new();
        let src = File::open(path)?;
        let reader = BufReader::new(src);
        'line: for line in reader.lines() {
            let line = line?;
            let mut items = line.split_ascii_whitespace();
            let phrase = items.next().unwrap();
            let _freq = items.next().unwrap();
            let mut bopomofos = Vec::new();
            for phones in items {
                for c in phones.chars() {
                    match Bopomofo::try_from(c) {
                        Ok(bopomofo) => bopomofos.push(bopomofo),
                        Err(_) => {
                            println!("Unknown bopomofo: {}", c);
                            continue 'line;
                        }
                    }
                }
            }
            dict.insert(&bopomofos, phrase.to_owned());
        }
        eprintln!("size of Bopomofo: {}", std::mem::size_of::<Bopomofo>());
        eprintln!("size of Node: {}", std::mem::size_of::<Node>());
        Ok(dict)
    }

    fn insert(&mut self, bopomofos: &[Bopomofo], phrase: String) {
        let mut node = &mut self.root;
        let mut stem_cur = 0;
        let mut bopomofo_iter = bopomofos.iter();
        loop {
            let &bopomofo = match bopomofo_iter.next() {
                Some(b) => b,
                None => break,
            };
            if stem_cur < node.stem.len() {
                if node.stem[stem_cur] == bopomofo {
                    stem_cur += 1;
                    continue;
                } else {
                    break;
                }
            }
            node = match node.find(bopomofo) {
                Some(_) => {
                    stem_cur = 0;
                    node.find_mut(bopomofo).unwrap()
                }
                None => break,
            };
            dbg!(&node);
        }
        let bopomofos = bopomofo_iter.as_slice();
        // match (node, stem_cur, bopomofos.len()) {
        //     (node, 0, 0) => {}
        //     (node, _, 0) => {}
        // }
        for &bopomofo in bopomofos {
            if stem_cur < node.stem.len() {
                if node.stem[stem_cur] == bopomofo {
                    stem_cur += 1;
                    continue;
                }
            }
            if stem_cur < node.stem.len() {
                debug_assert_ne!(node.stem[stem_cur], bopomofo);
                // split stem
                // let (common, tail) = node.stem.split_at(stem_cur);
                // let (edge_start, tail) = tail.split_at(1);
                let tail = node.stem[stem_cur..].to_vec();
                debug_assert_eq!(tail[0], node.stem[stem_cur]);
                // let tail = &node.stem[stem_cur + 1..];
                let mut new_node = Box::new(Node::default());
                // self.nodes_created += 1;
                new_node.stem.extend_from_slice(&tail[1..]);
                mem::swap(&mut node.next, &mut new_node.next);
                mem::swap(&mut node.phrases, &mut new_node.phrases);
                // new_node.next = mem::replace(&mut node.next, new_node.next);
                // new_node.phrases = mem::replace(&mut node.phrases, new_node.phrases);
                node.link(tail[0], new_node);
                node.stem.truncate(stem_cur);
                node.link(bopomofo, Box::new(Node::default()));
                stem_cur = 0;
                node = node.find_mut(bopomofo).unwrap();
                continue;
            }
            if node.next.len() == 0 && node.phrases.len() == 0 {
                node.stem.push(bopomofo);
                stem_cur += 1;
                continue;
            }
            node = match node.find(bopomofo) {
                Some(_) => {
                    stem_cur = 0;
                    node.find_mut(bopomofo).unwrap()
                }
                None => {
                    node.link(bopomofo, Box::new(Node::default()));
                    node.find_mut(bopomofo).unwrap()
                }
            };
        }
        node.add_phrase(phrase);
    }

    pub fn lookup(&self, bopomofos: &[Bopomofo]) -> Option<impl Iterator<Item = &String>> {
        let mut node = &self.root;
        let mut stem_cur = 0;
        for &bopomofo in bopomofos {
            if stem_cur < node.stem.len() {
                if node.stem[stem_cur] == bopomofo {
                    stem_cur += 1;
                    continue;
                } else {
                    return None;
                }
            }
            node = match node.find(bopomofo) {
                Some(n) => n,
                None => return None,
            };
            dbg!(&node);
        }
        Some(node.phrases())
    }

    // pub fn count_internal_nodes(&self) -> usize {
    //     let mut stack = Vec::new();
    //     let mut node = &self.root;
    //     let mut count = 0;
    //     loop {
    //         let mut order = 0;
    //         for n in node.next.iter() {
    //             stack.push(n.1);
    //             order += 1;
    //         }
    //         if node.phrases.len() == 0 && order == 1 {
    //             count += 1;
    //         }
    //         if stack.len() == 0 {
    //             break;
    //         }
    //         node = stack.pop().as_ref().unwrap();
    //     }
    //     count
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration, Instant};

    // #[test]
    // fn load_dictionary() {
    //     let now = Instant::now();
    //     let dict = Dictionary::load("data/tsi.src").unwrap();
    //     println!("Elapsed: {}ms", now.elapsed().as_millis());
    //     dbg!(dict
    //         .lookup(&[Bopomofo::N, Bopomofo::I, Bopomofo::TONE3,])
    //         .unwrap()
    //         .collect::<Vec<_>>());
    // }

    #[test]
    fn insert() {
        let mut dict = Dictionary::new();
        dict.insert(
            &[
                Bopomofo::T,
                Bopomofo::I,
                Bopomofo::AN,
                Bopomofo::TONE1,
                Bopomofo::M,
                Bopomofo::A,
                Bopomofo::TONE3,
            ],
            "天馬".to_owned(),
        );
        dbg!(&dict);
        dict.insert(
            &[Bopomofo::T, Bopomofo::I, Bopomofo::AN, Bopomofo::TONE1],
            "天".to_owned(),
        );
        dbg!(&dict);
        assert_eq!(
            dict.lookup(&[Bopomofo::T, Bopomofo::I, Bopomofo::AN, Bopomofo::TONE1])
                .unwrap()
                .next()
                .unwrap(),
            "天"
        );
    }
}
