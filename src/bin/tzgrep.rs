use anyhow::anyhow;
use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tzgrep::tar_foreach;

fn tar_grep<R: Read>(regex: Regex, input: R, line_number: bool) -> anyhow::Result<()> {
    if line_number {
        tar_foreach(input, &mut |filename, line_num, line| {
            if regex.is_match(line) {
                print!("{filename}:{line_num}:{line}");
            }
        })?
    } else {
        tar_foreach(input, &mut |filename, _, line| {
            if regex.is_match(line) {
                print!("{filename}:{line}");
            }
        })?
    }
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// serch pattern
    regex: Regex,
    /// serch target. If not presented read from stdin.
    ///
    /// .tar, .tar.gz, .tar.bz2, .tar.xz, .tar.zst are supported
    file: Option<PathBuf>,
    /// print line number with output lines
    #[arg(short = 'n', long)]
    line_number: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    macro_rules! f {
        ($input:expr) => {
            tar_grep(cli.regex, $input, cli.line_number)
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
