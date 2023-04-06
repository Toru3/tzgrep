use anyhow::anyhow;
use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tzgrep::tar_foreach;

enum Pattern {
    Regex(Regex),
    FixedStrings(String),
}
impl Pattern {
    fn new(pattern: String, is_fixed_string: bool) -> Result<Self, regex::Error> {
        if is_fixed_string {
            Ok(Self::FixedStrings(pattern))
        } else {
            Ok(Self::Regex(Regex::new(&pattern)?))
        }
    }
    fn is_match(&self, text: &str) -> bool {
        match &self {
            Self::Regex(x) => x.is_match(text),
            Self::FixedStrings(x) => x == text,
        }
    }
}

fn tar_grep<R: Read>(regex: Pattern, input: R, line_number: bool) -> anyhow::Result<()> {
    if line_number {
        tar_foreach(input, &mut |filename, line_num, line| {
            let line = line.trim_end_matches('\n');
            let line = line.trim_end_matches('\r');
            if regex.is_match(line) {
                println!("{filename}:{line_num}:{line}");
            }
        })?
    } else {
        tar_foreach(input, &mut |filename, _, line| {
            let line = line.trim_end_matches('\n');
            let line = line.trim_end_matches('\r');
            if regex.is_match(line) {
                println!("{filename}:{line}");
            }
        })?
    }
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// search pattern [regular expression](https://crates.io/crates/regex)
    pattern: String,
    /// search target. If not presented read from stdin.
    ///
    /// .tar, .tar.gz, .tar.bz2, .tar.xz, .tar.zst are supported
    file: Option<PathBuf>,
    /// print line number with output lines
    #[arg(short = 'n', long)]
    line_number: bool,
    /// Asuume search pattern to be fixed string.
    #[arg(short = 'F', long)]
    fixed_string: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let pattern = Pattern::new(cli.pattern, cli.fixed_string)?;
    macro_rules! f {
        ($input:expr) => {
            tar_grep(pattern, $input, cli.line_number)
        };
    }
    if let Some(file) = cli.file {
        if !file.exists() {
            return Err(anyhow!("file NOT exists: {}", file.display()));
        }
        if file.is_dir() {
            return Err(anyhow!("{} is directory", file.display()));
        }
        let fd = BufReader::new(File::open(&file)?);
        match file.extension() {
            None => Err(anyhow!("there is no extension")),
            Some(ext) if ext == "tar" => f!(fd),
            Some(ext) if ext == "gz" || ext == "tgz" => {
                f!(flate2::bufread::GzDecoder::new(fd))
            }
            Some(ext) if ext == "bz2" || ext == "tbz" => {
                f!(bzip2::bufread::BzDecoder::new(fd))
            }
            Some(ext) if ext == "xz" || ext == "txz" => {
                f!(xz::bufread::XzDecoder::new(fd))
            }
            Some(ext) if ext == "zst" || ext == "tzst" => {
                f!(zstd::stream::read::Decoder::new(fd)?)
            }
            Some(ext) => Err(anyhow!("Unsupported filetype: {ext:?}")),
        }
    } else {
        f!(std::io::stdin().lock())
    }
}
