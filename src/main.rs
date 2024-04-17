use clap::Parser;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use tar::Archive;
use tokei::{Config, LanguageType};

#[derive(Parser)]
struct Opts {
    pub tar_filename: String,
}

#[derive(Serialize)]
struct SummaryEntry {
    language: Option<LanguageType>,
    pub blanks: u64,
    pub code: u64,
    pub comments: u64,
    pub files: Vec<String>,
    pub bytes: u64,
}

fn main() {
    let opts: Opts = Opts::parse();
    let file = File::open(opts.tar_filename).unwrap();
    let mut a = Archive::new(file);
    let mut language_stats = BTreeMap::<Option<LanguageType>, SummaryEntry>::new();
    let config = Config::default();

    for res in a.entries().unwrap() {
        let mut f = res.unwrap();
        if !f.header().entry_type().is_file() {
            continue;
        }
        let path = f.header().path().unwrap().to_path_buf();
        match LanguageType::from_path(&path, &config) {
            Some(language) => {
                let mut s = Vec::new();
                f.read_to_end(&mut s).unwrap();
                let stats = language.parse_from_slice(&s, &config);
                let entry = language_stats
                    .entry(Some(language))
                    .or_insert_with(|| SummaryEntry {
                        language: Some(language),
                        blanks: 0,
                        code: 0,
                        comments: 0,
                        files: Vec::new(),
                        bytes: 0,
                    });
                let sum = stats.summarise();
                entry.blanks += sum.blanks as u64;
                entry.code += sum.code as u64;
                entry.comments += sum.comments as u64;
                entry.files.push(path.to_string_lossy().to_string());
                entry.bytes += s.len() as u64;
            }
            None => {
                let entry = language_stats
                    .entry(None)
                    .or_insert_with(|| SummaryEntry {
                        language: None,
                        blanks: 0,
                        code: 0,
                        comments: 0,
                        files: Vec::new(),
                        bytes: 0,
                    });
                entry.files.push(path.to_string_lossy().to_string());
                entry.bytes += f.header().size().unwrap();
            }
        }
    }
    let mut out = std::io::stdout().lock();
    for lang in language_stats.values() {
        serde_json::to_writer(&mut out, &lang).unwrap();
        out.write_all(b"\n").unwrap();
    }
}