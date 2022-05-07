use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
};

use chewing::{
    dictionary::TrieDictionaryBuilder,
    zhuyin::{Bopomofo, Syllable},
};
use chrono::Utc;
use clap::{Arg, Command};
use miette::{
    Context, Diagnostic, IntoDiagnostic, MietteSpanContents, Result, SourceCode, SourceSpan,
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
    let name: String = m.value_of_t_or_exit("name");
    let copyright: String = m.value_of_t_or_exit("copyright");
    let license: String = m.value_of_t_or_exit("license");
    let version: String = m.value_of_t_or_exit("version");

    let mut trie_builder = TrieDictionaryBuilder::new();
    trie_builder.set_name(name);
    trie_builder.set_copyright(copyright);
    trie_builder.set_license(license);
    trie_builder.set_version(version);
    trie_builder.set_software(format!(
        "{} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    ));
    trie_builder.set_created_date(timestamp);

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
        trie_builder
            .insert(&syllables, phrase, freq)
            .map_err(|err| ParseError {
                src: NamedLine {
                    name: tsi_src.clone(),
                    src: line.clone(),
                    line: line_num,
                    column: 0,
                },
                err_span: 0.into(),
                source: Box::new(err),
            })?;
    }

    let database = File::create(&output)
        .into_diagnostic()
        .wrap_err("Unable to create database file")?;
    let mut writer = BufWriter::new(database);

    println!("Writing database file to {} ...", output);
    trie_builder
        .write(&mut writer)
        .into_diagnostic()
        .wrap_err("Unable to write database file")?;
    println!("Done.");

    let stats = trie_builder.statistics();
    println!("Statistics:");
    println!("* Node count: {}", stats.node_count);
    println!(
        "* Different sounding phrases: {}",
        stats.internal_leaf_count
    );
    println!("* Different sounding words: {}", stats.root_branch_count);
    println!("* Total number of words/phrases: {}", stats.leaf_node_count);
    println!("* Longest phrase length: {}", stats.max_height - 2);
    println!("* Average phrase length: {}", stats.avg_height);
    println!("* Max branch count: {}", stats.max_branch_count);
    println!("* Average branch count: {}", stats.avg_branch_count);

    Ok(())
}
