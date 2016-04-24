// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::ffi::OsStr;
use std::path::PathBuf;


// Filesystem Utilities -------------------------------------------------------
pub fn filter_dir<F: FnMut(String, PathBuf)>(
    path: &PathBuf,
    ext: &str,
    mut callback: F
) {
    if let Ok(listing) = fs::read_dir(path) {
        for entry in listing {
            if let Ok(entry) = entry {
                if entry.file_type().unwrap().is_file() {
                    let path = entry.path();
                    if path.extension().unwrap_or(OsStr::new("")) == ext {
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

