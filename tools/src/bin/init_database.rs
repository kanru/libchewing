use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter},
};

use chewing::{
    dictionary::TrieDictionaryBuilder,
    zhuyin::{Bopomofo, Syllable},
};
use clap::{Arg, Command};
use miette::{Diagnostic, IntoDiagnostic, MietteSpanContents, Result, SourceCode, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("Parsing tsi.src failed")]
#[diagnostic(help("the format should be <phrase> <frequency> <bopomofo syllables>"))]
struct ParseError {
    #[source_code]
    src: SingleLineSourceCode,

    #[label("here")]
    err_span: SourceSpan,

    #[source]
    source: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug)]
struct SingleLineSourceCode(String, String, usize);

impl SourceCode for SingleLineSourceCode {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        _context_lines_before: usize,
        _context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        Ok(Box::new(MietteSpanContents::new_named(
            self.0.clone(),
            self.1.as_bytes(),
            *span,
            self.2,
            0,
            1,
        )))
    }
}

fn main() -> Result<()> {
    let m = Command::new("init_database")
        .about("This program creates a new chewing phrase dictionary file.")
        .arg(Arg::new("phone.cin").required(true))
        .arg(Arg::new("tsi.src").required(true))
        .arg_required_else_help(true)
        .get_matches();

    let phone_cin: String = m.value_of_t_or_exit("phone.cin");
    let tsi_src: String = m.value_of_t_or_exit("tsi.src");

    let mut trie_builder = TrieDictionaryBuilder::new();
    let tsi = File::open(&tsi_src).into_diagnostic()?;
    let reader = BufReader::new(tsi);
    for (line_num, line) in reader.lines().enumerate() {
        let mut syllables = vec![];
        let line = line.into_diagnostic()?;
        let phrase = line.split_ascii_whitespace().nth(0).unwrap();
        let freq: u32 = line
            .split_ascii_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .into_diagnostic()?;
        for syllable_str in line.split_ascii_whitespace().skip(2) {
            let mut syllable_builder = Syllable::builder();
            if syllable_str.starts_with('#') {
                break;
            }
            for c in syllable_str.chars() {
                syllable_builder =
                    syllable_builder.insert(Bopomofo::try_from(c).map_err(|err| ParseError {
                        src: SingleLineSourceCode(tsi_src.clone(), line.to_string(), line_num),
                        err_span: (0..line.len()).into(),
                        source: Box::new(err),
                    })?);
            }
            syllables.push(syllable_builder.build());
        }
        trie_builder.insert(&syllables, phrase, freq)?;
    }

    let database = File::create("chewing.dat").into_diagnostic()?;
    let mut writer = BufWriter::new(database);

    println!("Writing database file...");
    trie_builder.write(&mut writer).into_diagnostic()?;
    println!("Done.");

    let stats = trie_builder.statistics();
    println!("Statistics:");
    println!("* Node count: {}", stats.node_count);
    println!(
        "* Different sounding phrases: {}",
        stats.internal_leaf_count
    );
    println!("* Total number of phrases: {}", stats.leaf_node_count);
    println!("* Longest phrase length: {}", stats.max_height);
    println!("* Average phrase length: {}", stats.avg_height);
    println!("* Number of words: {}", stats.root_branch_count);
    println!("* Max branch count: {}", stats.max_branch_count);
    println!("* Average branch count: {}", stats.avg_branch_count);

    Ok(())
}
