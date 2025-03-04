// Conserve backup system.
// Copyright 2015, 2016, 2017, 2018 Martin Pool.

//! IO utilities.

use std::fs;
use std::io;
use std::io::prelude::*;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use tempfile;

use super::*;

pub struct AtomicFile {
    path: PathBuf,
    f: tempfile::NamedTempFile,
}

impl AtomicFile {
    pub fn new(path: &Path) -> Result<AtomicFile> {
        let dir = path.parent().unwrap();
        Ok(AtomicFile {
            path: path.to_path_buf(),
            f: tempfile::Builder::new().prefix("tmp").tempfile_in(dir)?,
        })
    }

    pub fn close(self, _report: &Report) -> Result<()> {
        // We use `persist` rather than `persist_noclobber` here because the latter calls
        // `link` on Unix, and some filesystems don't support it.  That's probably fine
        // because the files being updated by this should never already exist, though
        // it does mean we won't detect unexpected cases where it does.
        if let Err(e) = self.f.persist(&self.path) {
            return Err(e.error.into());
        };
        Ok(())
    }
}

impl Write for AtomicFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.f.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.f.flush()
    }
}

impl Deref for AtomicFile {
    type Target = fs::File;

    fn deref(&self) -> &Self::Target {
        self.f.as_file()
    }
}

impl DerefMut for AtomicFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.f.as_file_mut()
    }
}

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if let Err(e) = fs::create_dir(path) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    }
    Ok(())
}

/// True if path exists and is a directory, false if does not exist, error otherwise.
#[allow(dead_code)]
pub fn directory_exists(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(true)
            } else {
                Err(Error::NotADirectory(path.into()))
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(false),
            _ => Err(e.into()),
        },
    }
}

/// True if path exists and is a file, false if does not exist, error otherwise.
pub fn file_exists(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                Ok(true)
            } else {
                Err(Error::NotAFile(path.into()))
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(false),
            _ => Err(e.into()),
        },
    }
}

/// Create a directory if it doesn't exist; if it does then assert it must be empty.
pub fn require_empty_directory(path: &Path) -> Result<()> {
    if let Err(e) = std::fs::create_dir(&path) {
        if e.kind() == io::ErrorKind::AlreadyExists {
            // Exists and hopefully empty?
            if std::fs::read_dir(&path)?.next().is_some() {
                Err(Error::DestinationNotEmpty(path.into()))
            } else {
                Ok(()) // Exists and empty
            }
        } else {
            Err(Error::IoError(e))
        }
    } else {
        Ok(()) // Created
    }
}

/// List a directory.
///
/// Returns a list of filenames and a list of directory names respectively, forced to UTF-8, and
/// sorted naively as UTF-8.
pub fn list_dir(path: &Path) -> Result<(Vec<String>, Vec<String>)> {
    let mut file_names = Vec::<String>::new();
    let mut dir_names = Vec::<String>::new();
    for entry in fs::read_dir(path)? {
        let entry = entry.unwrap();
        let entry_filename = entry.file_name().into_string().unwrap();
        let entry_type = entry.file_type()?;
        if entry_type.is_file() {
            file_names.push(entry_filename);
        } else if entry_type.is_dir() {
            dir_names.push(entry_filename);
        } else {
            panic!("don't recognize file type of {:?}", entry_filename);
        }
    }
    file_names.sort_unstable();
    dir_names.sort_unstable();
    Ok((file_names, dir_names))
}

#[cfg(test)]
mod tests {
    // TODO: Somehow test the error cases.
    // TODO: Specific test for write_compressed_bytes.
}
