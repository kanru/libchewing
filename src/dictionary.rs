//! Dictionaries for looking up phrases.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
    path::Path,
};

use indexmap::IndexMap;
use miette::Diagnostic;
use thiserror::Error;

use crate::zhuyin::Syllable;

pub use sqlite::{SqliteDictionary, SqliteDictionaryBuilder, SqliteDictionaryError};
pub use trie::{TrieDictionary, TrieDictionaryBuilder, TrieDictionaryStatistics};

/// cbindgen:ignore
mod sqlite;
/// cbindgen:ignore
mod trie;

/// The error type which is returned from updating a dictionary.
#[derive(Error, Diagnostic, Debug)]
#[error("update dictionary failed")]
#[diagnostic(code(chewing::dictionary_update_error))]
pub struct DictionaryUpdateError {
    #[from]
    pub source: Box<dyn std::error::Error + Send + Sync>,
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

/// A type containing a phrase string and its frequency.
///
/// # Examples
///
/// A `Phrase` can be created from/to a tuple.
///
/// ```
/// use chewing::dictionary::Phrase;
///
/// let phrase = Phrase::new("測", 1);
/// assert_eq!(phrase, ("測", 1).into());
/// assert_eq!(("測".to_string(), 1u32), phrase.into());
/// ```
///
/// Phrases are ordered by their frequency.
///
/// ```
/// use chewing::dictionary::Phrase;
///
/// assert!(Phrase::new("測", 100) > Phrase::new("冊", 1));
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Phrase {
    phrase: String,
    freq: u32,
}

impl Phrase {
    /// Creates a new `Phrase`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("新", 1);
    /// ```
    pub fn new<S: Into<String>>(phrase: S, freq: u32) -> Phrase {
        Phrase {
            phrase: phrase.into(),
            freq,
        }
    }
    /// Returns the frequency of the phrase.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("詞頻", 100);
    ///
    /// assert_eq!(100, phrase.freq());
    /// ```
    pub fn freq(&self) -> u32 {
        self.freq
    }
    /// Returns the inner str of the phrase.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::Phrase;
    ///
    /// let phrase = Phrase::new("詞", 100);
    ///
    /// assert_eq!("詞", phrase.as_str());
    /// ```
    pub fn as_str(&self) -> &str {
        self.phrase.as_str()
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl PartialOrd for Phrase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.freq.partial_cmp(&other.freq) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.phrase.partial_cmp(&other.phrase)
    }
}

/// Phrases are compared by their frequency first, followed by their phrase
/// string.
impl Ord for Phrase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl AsRef<str> for Phrase {
    fn as_ref(&self) -> &str {
        self.phrase.as_str()
    }
}

impl From<Phrase> for String {
    fn from(phrase: Phrase) -> Self {
        phrase.phrase
    }
}

impl From<Phrase> for (String, u32) {
    fn from(phrase: Phrase) -> Self {
        (phrase.phrase, phrase.freq)
    }
}

impl<S> From<(S, u32)> for Phrase
where
    S: Into<String>,
{
    fn from(tuple: (S, u32)) -> Self {
        Phrase::new(tuple.0, tuple.1)
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.phrase.as_str())
    }
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
///     (vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], vec![("測", 100).into()]),
/// ]);
///
/// for phrase in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// ```
pub type Phrases<'a> = Box<dyn Iterator<Item = Phrase> + 'a>;

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
/// let dict_mut = dict.as_mut_dict().unwrap();
/// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
///
/// for phrase in dict.lookup_word(
///     syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]
/// ) {
///     assert_eq!("測", phrase.as_str());
///     assert_eq!(100, phrase.freq());
/// }
/// # Ok(())
/// # }
/// ```
pub trait Dictionary {
    /// Returns an iterator to all single syllable words matched by the
    /// syllable, if any. The result should use a stable order each time for the
    /// same input.
    fn lookup_word(&self, syllable: Syllable) -> Phrases {
        self.lookup_phrase(&[syllable])
    }
    /// Returns an iterator to all phrases matched by the syllables, if any. The
    /// result should use a stable order each time for the same input.
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases;
    /// Returns information about the dictionary instance.
    fn about(&self) -> DictionaryInfo;
    /// Returns a mutable reference to the dictionary if the underlying
    /// implementation allows update.
    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut>;
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
/// let dict_mut = dict.as_mut_dict().unwrap();
/// dict_mut.insert(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]], ("測", 100).into())?;
/// # Ok(())
/// # }
/// ```
pub trait DictionaryMut {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError>;

    fn update(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
        user_freq: u32,
    ) -> Result<(), DictionaryUpdateError>;
}

#[derive(Error, Debug, Diagnostic)]
#[error("build dictionary error")]
#[diagnostic(code(chewing::build_dictionary_error))]
pub struct BuildDictionaryError {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl From<std::io::Error> for BuildDictionaryError {
    fn from(source: std::io::Error) -> Self {
        BuildDictionaryError {
            source: Box::new(source),
        }
    }
}

pub trait DictionaryBuilder {
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError>;
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError>;
    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError>;
}

impl Dictionary for HashMap<Vec<Syllable>, Vec<Phrase>> {
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases {
        self.get(syllables)
            .cloned()
            .map(|v| Box::new(v.into_iter()) as Phrases)
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }

    fn about(&self) -> DictionaryInfo {
        Default::default()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for HashMap<Vec<Syllable>, Vec<Phrase>> {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let vec = self.entry(syllables.to_vec()).or_default();
        if vec.iter().any(|it| it.as_str() == phrase.as_str()) {
            return Err(DictionaryUpdateError {
                source: Box::new(DuplicatePhraseError),
            });
        }
        vec.push(phrase);
        Ok(())
    }

    fn update(
        &mut self,
        _syllables: &[Syllable],
        _phrase: Phrase,
        _user_freq: u32,
    ) -> Result<(), DictionaryUpdateError> {
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
///     vec![("測", 1).into(), ("冊", 1).into(), ("側", 1).into()]
/// );
/// user_dict.insert(
///     vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
///     vec![("策", 100).into(), ("冊", 100).into()]
/// );
///
/// let user_block_list = Box::new(HashSet::from(["側".to_string()]));
///
/// let dict = ChainedDictionary::new(vec![sys_dict, user_dict], vec![user_block_list]);
/// assert_eq!(
///     [
///         ("策", 100).into(),
///         ("冊", 100).into(),
///         ("測", 1).into(),
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
                // Use IndexMap so the insertion order is preserved
                dict.lookup_phrase(syllables)
                    .map(|phrase| phrase.into())
                    .collect::<IndexMap<String, u32>>()
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
                |h| {
                    Box::new(
                        h.into_iter()
                            .filter(|(phrase, _)| !self.is_blocked(phrase))
                            .map(|v| v.into()),
                    )
                },
            )
    }

    fn about(&self) -> DictionaryInfo {
        DictionaryInfo {
            name: Some("Built-in ChainedDictionary".to_string()),
            ..Default::default()
        }
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        Some(self)
    }
}

impl DictionaryMut for ChainedDictionary {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            if let Some(dict_mut) = dict.as_mut_dict() {
                dict_mut.insert(syllables, phrase.clone())?;
            }
        }
        Ok(())
    }

    fn update(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
        user_freq: u32,
    ) -> Result<(), DictionaryUpdateError> {
        for dict in &mut self.inner {
            if let Some(dict_mut) = dict.as_mut_dict() {
                dict_mut.update(syllables, phrase.clone(), user_freq)?;
            }
        }
        Ok(())
    }
}
