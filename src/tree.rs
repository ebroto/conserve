// Conserve backup system.
// Copyright 2017, 2018, 2019 Martin Pool.

//! Abstract Tree trait.

use std::io::ErrorKind;
use std::ops::Range;

use crate::*;

/// Abstract Tree that may be either on the real filesystem or stored in an archive.
pub trait ReadTree: HasReport {
    type I: Iterator<Item = Result<Entry>>;
    type R: std::io::Read;

    fn iter_entries(&self, report: &Report) -> Result<Self::I>;

    /// Read file contents as a `std::io::Read`.
    ///
    /// This is softly deprecated in favor of `read_file_blocks`.
    fn file_contents(&self, entry: &Entry) -> Result<Self::R>;

    /// Estimate the number of entries in the tree.
    /// This might do somewhat expensive IO, so isn't the Iter's `size_hint`.
    fn estimate_count(&self) -> Result<u64>;

    /// Measure the tree size.
    ///
    /// This typically requires walking all entries, which may take a while.
    fn size(&self) -> Result<TreeSize> {
        let report = self.report();
        let mut tot = 0u64;
        for e in self.iter_entries(self.report())? {
            // While just measuring size, ignore directories/files we can't stat.
            match e {
                Ok(e) => {
                    let s = e.size().unwrap_or(0);
                    tot += s;
                    report.increment_work(s);
                }
                Err(Error::IoError(ioe)) => match ioe.kind() {
                    // Fairly harmless errors to encounter while walking a tree; can be ignored
                    // while computing the size.
                    ErrorKind::NotFound | ErrorKind::PermissionDenied => (),
                    // May be serious?
                    _ => return Err(Error::IoError(ioe)),
                },
                Err(err) => return Err(err),
            }
        }
        Ok(TreeSize { file_bytes: tot })
    }
}

/// A tree open for writing, either local or an an archive.
///
/// This isn't a sub-trait of ReadTree since a backup band can't be read while writing is
/// still underway.
///
/// Entries must be written in Apath order, since that's a requirement of the index.
pub trait WriteTree {
    fn finish(&mut self) -> Result<()>;

    fn write_dir(&mut self, entry: &Entry) -> Result<()>;
    fn write_symlink(&mut self, entry: &Entry) -> Result<()>;
    fn write_file(&mut self, entry: &Entry, content: &mut dyn std::io::Read) -> Result<()>;

    /// Copy in the contents of a file from another tree.
    fn copy_file<R: ReadTree>(&mut self, entry: &Entry, from_tree: &R) -> Result<()> {
        let mut content = from_tree.file_contents(&entry)?;
        // TODO(#69): Rather than always writing the content, check if it's changed versus
        // a reference tree. (Should that be a parameter to this method, or tracked by the
        // WriteTree?)
        //
        // If it has changed, copy the content as usual. If not, tell the WriteTree that
        // it's unchanged, which can be handled in a tree-specific way. So, this probably
        // means the WriteTree can pass an object.
        //
        // The BackupWriter already is stateful and visits files in order, so perhaps
        // it's fine to have it internally hold an iterator over the reference tree(s).
        //
        // Then, instead of passing back an object saying whether it's new or not, we
        // could potentially pass in the stat information and give the WriteTree a
        // chance to mark the file as unchanged. If it is changed, then we need to get
        // the content from this file (preferably as a `Read`) and store that...
        self.write_file(entry, &mut content)
    }
}

/// Read a file as a series of blocks of bytes.
///
/// When reading from the archive, the blocks are whatever size is stored.
/// When reading from the filesystem they're MAX_BLOCK_SIZE. But the caller
/// shouldn't assume the size.
pub trait ReadBlocks {
    /// Return a range of integers indexing the blocks (starting from 0.)
    fn num_blocks(&self) -> Result<usize>;

    fn block_range(&self) -> Result<Range<usize>> {
        Ok(0..self.num_blocks()?)
    }

    fn read_block(&self, i: usize) -> Result<Vec<u8>>;
}

/// The measured size of a tree.
pub struct TreeSize {
    pub file_bytes: u64,
}
