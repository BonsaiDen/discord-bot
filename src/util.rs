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
use hyper::header::Connection;


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

pub fn download_file(mut directory: PathBuf, ext: &str, name: &str, url: &str) -> Result<(), ()> {

    directory.push(name);
    directory.set_extension(ext);

    // TODO clean up

    let client = Client::new();
    if let Ok(mut res) = client.get(url).header(Connection::close()).send() {
        let mut buffer = Vec::new();
        match File::create(directory) {
            Ok(mut f) => {
                match res.read_to_end(&mut buffer) {
                    Ok(_) => {
                        info!("[Download] Failed to download file from url: {}", url);
                        match f.write_all(&buffer) {
                            Ok(_) => Ok(()),
                            Err(err) => {
                                info!("[Download] Failed to write to download to file: {}", err);
                                Err(())
                            }
                        }
                    }
                    Err(err) => {
                        info!("[Download] Failed to download file from url: {}", err);
                        Err(())
                    }
                }
            }
            Err(err) => {
                info!("[Download] Failed to create download file: {}", err);
                Err(())
            }
        }

    } else {
        Err(())
    }

}


// Listing Utilities ----------------------------------------------------------
pub fn list_words<'a>(
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

pub fn list_lines<'a>(
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

