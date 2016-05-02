// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::cmp;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::ffi::OsStr;
use std::path::PathBuf;


// External Dependencies ------------------------------------------------------
use hyper::Client;
use hyper::header::{Connection, Range, ByteRangeSpec, ContentLength};
use flac::{ByteStream, Stream};
use flac::metadata::StreamInfo;


// Filesystem Utilities -------------------------------------------------------
pub fn filter_dir<F: FnMut(String, PathBuf)>(
    path: PathBuf,
    ext: &str,
    mut callback: F
) {
    if let Ok(listing) = fs::read_dir(path) {
        for entry in listing {
            if let Ok(entry) = entry {
                if entry.file_type().unwrap().is_file() {
                    let path = entry.path();
                    if path.extension().unwrap_or_else(||OsStr::new("")) == ext {
                        if let Some(stem) = path.file_stem() {
                            if stem != "" {
                                callback(
                                    stem.to_str().unwrap().to_string(),
                                    PathBuf::from(path.clone())
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn download_file(
    mut directory: PathBuf,
    name: &str,
    ext: &str,
    url: &str

) -> Result<(), String> {

    directory.push(name);
    directory.set_extension(ext);

    let client = Client::new();
    client.get(url)
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut resp| {
            let mut buffer = Vec::<u8>::new();
            resp.read_to_end(&mut buffer)
                .map_err(|err| err.to_string())
                .map(|_| buffer)
        })
        .and_then(|buffer| {
            File::create(directory)
                .map_err(|err| err.to_string())
                .and_then(|mut file| {
                    file.write_all(&buffer)
                        .map_err(|err| err.to_string())
                })
        })

}

pub fn delete_file(
    mut directory: PathBuf,
    name: &str,
    ext: &str,

) -> Result<(), String> {
    directory.push(name);
    directory.set_extension(ext);
    fs::remove_file(directory).map_err(|err| err.to_string())
}

pub fn retrieve_flac_info(url: &str) -> Result<(u64, StreamInfo), String> {

    let client = Client::new();
    client.get(url)
        .header(Range::Bytes(vec![ByteRangeSpec::FromTo(0, 256)]))
        .header(Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut resp| {
            let length = resp.headers.get::<ContentLength>().map_or(0, |l| l.0);
            let mut header = Vec::new();
            resp.read_to_end(&mut header)
                .map_err(|err| err.to_string())
                .map(|_| (length, header))
        })
        .and_then(|(length, header)| {
            Stream::<ByteStream>::from_buffer(&header[..])
                .map_err(|_| "Failed to parse flac header.".to_string())
                .map(|stream| (length, stream.info()))
        })

}


// Listing Utilities ----------------------------------------------------------
pub fn list_words(
    title: &str,
    words: Vec<&str>,
    block_size: usize,
    line_size: usize

) -> Vec<String> {

    let total = words.len();
    words.chunks(block_size).enumerate().map(|(index, block)| {

        let lines: Vec<String> = block.chunks(line_size).map(|c| {
            c.join(", ")

        }).collect();

        let offset = index * block_size + 1;
        format!(
            "\n__{} {} - {} of {}:__\n\n - {}",
            title,
            offset,
            cmp::min(offset + (block_size - 1), total),
            total,
            lines.join("\n - ")
        )

    }).collect()

}

pub fn list_lines(
    title: &str,
    lines: Vec<String>,
    line_size: usize

) -> Vec<String> {

    let total = lines.len();
    lines.chunks(line_size).enumerate().map(|(index, lines)| {

        let offset = index * line_size + 1;
        format!(
            "\n__{} {} - {} of {}:__\n\n - {}",
            title,
            offset,
            cmp::min(offset + (line_size - 1), total),
            total,
            lines.join("\n - ")
        )

    }).collect()

}

