use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{bail, Context, Result};
use chewing::{
    dictionary::{
        DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder, TrieDictionaryBuilder,
    },
    zhuyin::{Bopomofo, Syllable},
};
use clap::{Arg, Command};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Error, Debug)]
#[error("parsing failed at line {line_num}")]
struct ParseError {
    line_num: usize,
    column: usize,
    #[source]
    source: anyhow::Error,
}

trait IntoParseError<T> {
    fn parse_error(self, line_num: usize, column: usize) -> std::result::Result<T, ParseError>;
}

impl<T> IntoParseError<T> for Result<T> {
    fn parse_error(self, line_num: usize, column: usize) -> std::result::Result<T, ParseError> {
        self.map_err(|source| ParseError {
            line_num, column, source
        })
    }
}

fn main() -> Result<()> {
    let today = OffsetDateTime::now_utc().date();
    let timestamp = today.to_string();
    let m = Command::new("init_database")
        .about("This program creates a new chewing phrase dictionary file.")
        .arg(
            Arg::new("type")
                .short('t')
                .takes_value(true)
                .possible_value("sqlite")
                .possible_value("trie")
                .default_value("sqlite"),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .takes_value(true)
                .default_value("我的詞庫"),
        )
        .arg(
            Arg::new("copyright")
                .short('c')
                .takes_value(true)
                .default_value("Unknown"),
        )
        .arg(
            Arg::new("license")
                .short('l')
                .takes_value(true)
                .default_value("Unknown"),
        )
        .arg(
            Arg::new("version")
                .short('r')
                .takes_value(true)
                .default_value(&timestamp),
        )
        .arg(Arg::new("tsi.src").required(true))
        .arg(Arg::new("output").required(true))
        .arg_required_else_help(true)
        .get_matches();

    let tsi_src: String = m.value_of_t_or_exit("tsi.src");
    let output: String = m.value_of_t_or_exit("output");
    let db_type: String = m.value_of_t_or_exit("type");
    let name: String = m.value_of_t_or_exit("name");
    let copyright: String = m.value_of_t_or_exit("copyright");
    let license: String = m.value_of_t_or_exit("license");
    let version: String = m.value_of_t_or_exit("version");

    let mut builder: Box<dyn DictionaryBuilder> = match db_type.as_str() {
        "sqlite" => Box::new(SqliteDictionaryBuilder::new()),
        "trie" => Box::new(TrieDictionaryBuilder::new()),
        _ => bail!("Unknown database type {}", db_type),
    };

    builder.set_info(DictionaryInfo {
        name: name.into(),
        copyright: copyright.into(),
        license: license.into(),
        version: version.into(),
        software: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).into(),
        created_date: timestamp.into(),
    })?;

    let tsi = File::open(&tsi_src)?;
    let reader = BufReader::new(tsi);
    for (line_num, line) in reader.lines().enumerate() {
        let mut syllables = vec![];
        let line = line?;
        let phrase = line.split_ascii_whitespace().next().unwrap();
        let freq: u32 = line
            .split_ascii_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .context("unable to parse frequency")
            .parse_error(line_num, 0)?;
        for syllable_str in line.split_ascii_whitespace().skip(2) {
            let mut syllable_builder = Syllable::builder();
            if syllable_str.starts_with('#') {
                break;
            }
            for c in syllable_str.chars() {
                syllable_builder =
                    syllable_builder.insert(Bopomofo::try_from(c)?);
            }
            syllables.push(syllable_builder.build());
        }
        builder
            .insert(&syllables, (phrase, freq).into())?;
    }
    let path: &Path = output.as_ref();
    if path.exists() {
        fs::remove_file(path)
            .context("unable to overwrite output")?;
    }
    builder.build(path)?;

    Ok(())
}
