//! Dictionaries for looking up phrases.

use std::collections::{HashMap, HashSet};

use miette::Diagnostic;
use thiserror::Error;

use crate::zhuyin::Syllable;

pub use trie::{TrieDictionary, TrieDictionaryBuilder, TrieDictionaryStatistics};

/// cbindgen:ignore
mod trie;

/// The error type which is returned from updating a dictionary.
#[derive(Error, Diagnostic, Debug)]
#[error("update dictionary failed")]
#[diagnostic(code(chewing::dictionary_update_error))]
pub struct DictionaryUpdateError {
    pub source: Box<dyn std::error::Error>,
}

/// The error type which is returned from building or updating a dictionary.
#[derive(Error, Diagnostic, Debug)]
#[error("found duplicated phrases")]
#[diagnostic(code(chewing::duplicate_phrase_error))]
pub struct DuplicatePhraseError;

/// A collection of metadata of a dictionary.
///
/// The dictionary version and copyright information can be used in
/// configuration application.
///
/// # Examples
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use chewing::dictionary::Dictionary;
/// # let dictionary = HashMap::new();
/// let about = dictionary.about();
/// assert_eq!("libchewing default", about.name.unwrap());
/// assert_eq!("Copyright (c) 2022 libchewing Core Team", about.copyright.unwrap());
/// assert_eq!("LGPL-2.1-or-later", about.license.unwrap());
/// assert_eq!("init_database 0.5.1", about.software.unwrap());
/// ```
#[derive(Clone, Default)]
pub struct DictionaryInfo {
    /// The name of the dictionary.
    pub name: Option<String>,
    /// The copyright information of the dictionary.
    ///
    /// It's recommended to include the copyright holders' names and email
    /// addresses, separated by semicolons.
    pub copyright: Option<String>,
    /// The license information of the dictionary.
    ///
    /// It's recommended to use the [SPDX license identifier](https://spdx.org/licenses/).
    pub license: Option<String>,
    /// The date the dictionary was created
    ///
    /// It's recommended to use the ISO-8601 format, YYYY-MM-DD.
    pub created_date: Option<String>,
    /// The version of the dictionary.
    ///
    /// It's recommended to use the commit hash or revision if the dictionary is
    /// managed in a source control repository.
    pub version: Option<String>,
    /// The name of the software used to generate the dictionary.
    ///
    /// It's recommended to include the name and the version number.
    pub software: Option<String>,
}

/// A generic iterator over the phrases and their frequency in a dictionary.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
///
/// let dict = HashMap::from([
///     (vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], vec![("測".to_string(), 100)]),
/// ]);
///
/// for (phrase, freq) in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase);
///     assert_eq!(100, freq);
/// }
/// ```
pub type Phrases<'a> = Box<dyn Iterator<Item = (String, u32)> + 'a>;

/// An interface for looking up dictionaries.
///
/// This is the main dictionary trait. For more about the concept of
/// dictionaries generally, please see the [module-level
/// documentation][crate::dictionary].
///
/// # Examples
///
/// The std [`HashMap`] implements the `Dictionary` trait so it can be used in
/// tests.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::HashMap;
///
/// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
///
/// let mut dict = HashMap::new();
/// let dict_mut = dict.as_dict_mut().unwrap();
/// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], "測", 100)?;
///
/// for (phrase, freq) in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase);
///     assert_eq!(100, freq);
/// }
/// # Ok(())
/// # }
/// ```
pub trait Dictionary {
    /// Returns an iterator to all single syllable words matched by the
    /// syllable, if any. The result does not guarantee any sorting order.
    fn lookup_word(&self, syllable: Syllable) -> Phrases {
        self.lookup_phrase(&[syllable])
    }
    /// Returns an iterator to all phrases matched by the syllables, if any. The
    /// result does not guarantee any sorting order.
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases;
    /// Returns information about the dictionary instance.
    fn about(&self) -> DictionaryInfo;
    /// Returns a mutable reference to the dictionary if the underlying
    /// implementation allows update.
    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut>;
}

/// An interface for updating dictionaries.
///
/// For more about the concept of dictionaries generally, please see the
/// [module-level documentation][crate::dictionary].
///
/// # Examples
///
/// The std [`HashMap`] implements the `DictionaryMut` trait so it can be used in
/// tests.
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::HashMap;
///
/// use chewing::{dictionary::Dictionary, syl, zhuyin::Bopomofo};
///
/// let mut dict = HashMap::new();
/// let dict_mut = dict.as_dict_mut().unwrap();
/// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], "測", 100)?;
/// # Ok(())
/// # }
/// ```
pub trait DictionaryMut {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
        frequency: u32,
    ) -> Result<(), DictionaryUpdateError>;
}

impl Dictionary for HashMap<Vec<Syllable>, Vec<(String, u32)>> {
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases {
        self.get(syllables)
            .cloned()
            .map(|v| Box::new(v.into_iter()) as Phrases)
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }

    fn about(&self) -> DictionaryInfo {
        Default::default()
    }

    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for HashMap<Vec<Syllable>, Vec<(String, u32)>> {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
        frequency: u32,
    ) -> Result<(), DictionaryUpdateError> {
        let vec = self.entry(syllables.to_vec()).or_default();
        if vec.iter().any(|it| it.0 == phrase) {
            return Err(DictionaryUpdateError {
                source: Box::new(DuplicatePhraseError),
            });
        }
        vec.push((phrase.to_owned(), frequency));
        Ok(())
    }
}

/// A block list contains unwanted phrases.
pub trait BlockList {
    /// Returns if whether a phrase is in the block list.
    fn is_blocked(&self, phrase: &str) -> bool;
}

impl BlockList for HashSet<String> {
    fn is_blocked(&self, phrase: &str) -> bool {
        self.contains(phrase)
    }
}

/// A collection of dictionaries that returns the union of the lookup results.
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::collections::{HashMap, HashSet};
///
/// use chewing::{dictionary::{ChainedDictionary, Dictionary}, syl, zhuyin::Bopomofo};
///
/// let mut sys_dict = Box::new(HashMap::new());
/// let mut user_dict = Box::new(HashMap::new());
/// sys_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("測".to_string(), 1), ("冊".to_string(), 1), ("側".to_string(), 1)]
/// );
/// user_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策".to_string(), 100), ("冊".to_string(), 100)]
/// );
///
/// let user_block_list = Box::new(HashSet::from(["側".to_string()]));
///
/// let dict = ChainedDictionary::new(vec![sys_dict, user_dict], vec![user_block_list]);
/// assert_eq!(
///     [
///         ("策".to_string(), 100),
///         ("冊".to_string(), 100),
///         ("測".to_string(), 1),
///     ]
///     .into_iter()
///     .collect::<HashSet<_>>(),
///     dict.lookup_phrase(&[
///         syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
///     ])
///     .collect::<HashSet<_>>(),
/// );
/// # Ok(())
/// # }
/// ```
pub struct ChainedDictionary {
    inner: Vec<Box<dyn Dictionary>>,
    blocked: Vec<Box<dyn BlockList>>,
}

impl ChainedDictionary {
    /// Creates a new `ChainedDictionary` with the list of dictionaries and
    /// block lists.
    pub fn new(
        dictionaries: Vec<Box<dyn Dictionary>>,
        block_lists: Vec<Box<dyn BlockList>>,
    ) -> ChainedDictionary {
        ChainedDictionary {
            inner: dictionaries,
            blocked: block_lists,
        }
    }
    fn is_blocked(&self, phrase: &str) -> bool {
        self.blocked.iter().any(|b| b.is_blocked(phrase))
    }
}

impl Dictionary for ChainedDictionary {
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases {
        self.inner
            .iter()
            .map(|dict| {
                dict.lookup_phrase(syllables)
                    .collect::<HashMap<String, u32>>()
            })
            .reduce(|mut accum, second| {
                for (phrase, mut freq) in second.into_iter() {
                    let entry = accum.entry(phrase).or_default();
                    *entry = *entry.max(&mut freq);
                }
                accum
            })
            .map_or_else(
                || Box::new(std::iter::empty()) as Phrases,
                |h| Box::new(h.into_iter().filter(|(phrase, _)| !self.is_blocked(phrase))),
            )
    }

    fn about(&self) -> DictionaryInfo {
        DictionaryInfo {
            name: Some("Built-in ChainedDictionary".to_string()),
            ..Default::default()
        }
    }

    fn as_dict_mut(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for ChainedDictionary {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: &str,
        frequency: u32,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            if let Some(dict_mut) = dict.as_dict_mut() {
                dict_mut.insert(syllables, phrase, frequency)?;
            }
        }
        Ok(())
    }
}
