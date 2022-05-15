use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use chewing::{
    dictionary::{
        DictionaryBuilder, DictionaryInfo, SqliteDictionaryBuilder, TrieDictionaryBuilder,
    },
    zhuyin::{Bopomofo, Syllable},
};
use chrono::Utc;
use clap::{Arg, Command};
use miette::{
    bail, Context, Diagnostic, IntoDiagnostic, MietteSpanContents, Result, SourceCode, SourceSpan,
};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("Parsing tsi.src failed")]
#[diagnostic(help("the format should be <phrase> <frequency> <bopomofo syllables>"))]
struct ParseError {
    #[source_code]
    src: NamedLine,

    #[label("here")]
    err_span: SourceSpan,

    #[source]
    source: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug)]
struct NamedLine {
    name: String,
    src: String,
    line: usize,
    column: usize,
}

impl NamedLine {
    fn new(name: String, src: String, line: usize, column: usize) -> NamedLine {
        NamedLine {
            name,
            src,
            line,
            column,
        }
    }
}

impl SourceCode for NamedLine {
    fn read_span<'a>(
        &'a self,
        _span: &SourceSpan,
        _context_lines_before: usize,
        _context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        Ok(Box::new(MietteSpanContents::new_named(
            self.name.clone(),
            self.src.as_bytes(),
            (0..self.src.as_bytes().len()).into(),
            self.line,
            self.column,
            1,
        )))
    }
}

fn main() -> Result<()> {
    let timestamp = Utc::today().format("%Y-%m-%d").to_string();
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

    let tsi = File::open(&tsi_src).into_diagnostic()?;
    let reader = BufReader::new(tsi);
    for (line_num, line) in reader.lines().enumerate() {
        let mut syllables = vec![];
        let line = line.into_diagnostic()?;
        let phrase = line.split_ascii_whitespace().next().unwrap();
        let freq: u32 = line
            .split_ascii_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .map_err(|err| ParseError {
                src: NamedLine::new(tsi_src.clone(), line.to_string(), line_num, 0),
                err_span: 0.into(),
                source: Box::new(err),
            })
            .wrap_err("Unable to parse frequency")?;
        for syllable_str in line.split_ascii_whitespace().skip(2) {
            let mut syllable_builder = Syllable::builder();
            if syllable_str.starts_with('#') {
                break;
            }
            for c in syllable_str.chars() {
                syllable_builder =
                    syllable_builder.insert(Bopomofo::try_from(c).map_err(|err| ParseError {
                        src: NamedLine::new(tsi_src.clone(), line.to_string(), line_num, 0),
                        err_span: 0.into(),
                        source: Box::new(err),
                    })?);
            }
            syllables.push(syllable_builder.build());
        }
        builder
            .insert(&syllables, (phrase, freq).into())
            .map_err(|err| ParseError {
                src: NamedLine {
                    name: tsi_src.clone(),
                    src: line.clone(),
                    line: line_num,
                    column: 0,
                },
                err_span: 0.into(),
                source: err.into(),
            })?;
    }
    let path: &Path = output.as_ref();
    if path.exists() {
        fs::remove_file(path)
            .into_diagnostic()
            .wrap_err("Unable to overwrite output")?;
    }
    builder.build(path)?;

    Ok(())
}
