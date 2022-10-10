use itertools::Itertools;
use std::io::{BufRead, BufReader, Read};
use thiserror::Error;

const BLOCK_SIZE: usize = 512;

#[derive(Error, Debug)]
pub enum TzgrepError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("utf-8 convert error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("invalid octal: {0}")]
    OctalError(u8),
}

#[derive(Debug)]
#[non_exhaustive]
enum FileType {
    Regular,
    EndOfArchive,
    Other,
}

impl FileType {
    fn new(c: u8) -> Self {
        use FileType::*;
        match c {
            // NUL or '0'
            0x00 | 0x30 => Regular,
            _ => Other,
        }
    }
}

#[derive(Debug)]
struct Header<'a> {
    name: &'a str,
    size: u64,
    filetype: FileType,
}

impl<'a> Header<'a> {
    fn parse_str(buf: &[u8]) -> Result<&str, TzgrepError> {
        let end = buf.iter().find_position(|&&x| x == 0);
        if let Some((p, _)) = end {
            Ok(std::str::from_utf8(&buf[0..p])?)
        } else {
            Ok(std::str::from_utf8(buf)?)
        }
    }
    fn parse_octal(buf: &[u8]) -> Result<u64, TzgrepError> {
        let mut r = 0;
        for c in buf.iter() {
            if c & 0xF8 != 0x30 {
                return Err(TzgrepError::OctalError(*c));
            }
            r = r << 3 | (c & 0x07) as u64;
        }
        Ok(r)
    }
    fn new(buf: &'a [u8; BLOCK_SIZE]) -> Result<Self, TzgrepError> {
        let name = Self::parse_str(&buf[0..100])?;
        if name.is_empty() {
            return Ok(Self {
                name,
                size: 0,
                filetype: FileType::EndOfArchive,
            });
        }
        let size = Self::parse_octal(&buf[124..135])?;
        let filetype = FileType::new(buf[156]);
        Ok(Self {
            name,
            size,
            filetype,
        })
    }
}

fn num_blocks(size: usize) -> usize {
    (size + BLOCK_SIZE - 1) / BLOCK_SIZE
}

fn foreach_line<R: BufRead, F: FnMut(&str, usize, &str)>(filename: &str, mut input: R, func: &mut F) {
    let mut line = String::new();
    for line_number in 1.. {
        let r = input.read_line(&mut line);
        if let Ok(0) = r {
            return;
        } else if r.is_err() {
            break;
        }
        func(filename, line_number, &line);
        line.clear();
    }
    let mut buffer = [0; BLOCK_SIZE];
    loop {
        let r = input.read(&mut buffer);
        if let Ok(0) = r {
            return;
        } else if r.is_err() {
            break;
        }
    }
}

/// apply `func` for each line for each file in tar
///
/// `input` must be tar
/// `func` is called `func(filename, line_number, line_string)`
pub fn tar_foreach<R: Read, F: FnMut(&str, usize, &str)>(
    mut input: R,
    func: &mut F,
) -> Result<(), TzgrepError> {
    let mut buffer = [0; BLOCK_SIZE];
    loop {
        input.read_exact(&mut buffer)?;
        //println!("{:02X?}", buffer);
        let h = Header::new(&buffer)?;
        use FileType::*;
        match h.filetype {
            Regular => {
                let iw = BufReader::new(input.by_ref().take(h.size));
                foreach_line(h.name, iw, func);
                let size = h.size as usize;
                let remain = num_blocks(size) * BLOCK_SIZE - size;
                input.read_exact(&mut buffer[0..remain])?;
            }
            EndOfArchive => break,
            _ => {
                // discard
                for _ in 0..num_blocks(h.size as usize) {
                    input.read_exact(&mut buffer)?;
                }
            }
        }
    }
    Ok(())
}
