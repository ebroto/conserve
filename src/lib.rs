// Copyright 2015, 2016, 2017, 2018, 2019 Martin Pool.

//! Conserve backup system.

extern crate blake2_rfc;
extern crate chrono;
extern crate hex;
extern crate isatty;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate snap;
extern crate tempfile;
extern crate term;
extern crate terminal_size;
extern crate thousands;
extern crate unicode_segmentation;
extern crate walkdir;

#[cfg(test)]
extern crate spectral;

extern crate globset;

// Conserve implementation modules.
mod apath;
mod archive;
mod backup;
mod band;
mod bandid;
mod blockdir;
pub mod compress;
mod copy_tree;
mod entry;
pub mod errors;
pub mod excludes;
pub mod index;
mod io;
mod jsonio;
pub mod live_tree;
mod merge;
mod misc;
pub mod output;
pub mod report;
mod restore;
mod stored_file;
mod stored_tree;
pub mod test_fixtures;
mod tree;
pub mod ui;

pub use crate::apath::Apath;
pub use crate::archive::Archive;
pub use crate::backup::BackupWriter;
pub use crate::band::Band;
pub use crate::bandid::BandId;
pub use crate::blockdir::BlockDir;
pub use crate::compress::snappy::Snappy;
pub use crate::compress::Compression;
pub use crate::copy_tree::copy_tree;
pub use crate::entry::{Entry, Kind};
pub use crate::errors::*;
pub use crate::index::{IndexBuilder, ReadIndex};
pub use crate::io::{ensure_dir_exists, list_dir, AtomicFile};
pub use crate::live_tree::LiveTree;
pub use crate::merge::{iter_merged_entries, MergedEntryKind};
pub use crate::report::{HasReport, Report, Sizes};
pub use crate::restore::RestoreTree;
pub use crate::stored_tree::StoredTree;
pub use crate::tree::{ReadBlocks, ReadTree, TreeSize, WriteTree};
pub use crate::ui::UI;

// Commonly-used external types.
pub use globset::GlobSet;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn version() -> &'static str {
    VERSION
}

/// Format-compatibility version, normally the first two components of the package version.
///
/// (This might be older than the program version.)
pub const ARCHIVE_VERSION: &str = "0.6";

pub const SYMLINKS_SUPPORTED: bool = cfg!(target_family = "unix");

/// Break blocks at this many uncompressed bytes.
pub(crate) const MAX_BLOCK_SIZE: usize = 1 << 20;
