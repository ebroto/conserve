// Copyright 2017, 2018, 2019 Martin Pool.

//! Access a versioned tree stored in the archive.
//!
//! Through this interface you can iterate the contents and retrieve file contents.
//!
//! This is the preferred higher-level interface for reading stored versions. It'll abstract
//! across incremental backups, hiding from the caller that data may be distributed across
//! multiple index files, bands, and blocks.

use rayon::iter::ParallelBridge;
use rayon::prelude::*;

use crate::stored_file::{ReadStoredFile, StoredFile};
use crate::*;

/// Read index and file contents for a version stored in the archive.
#[derive(Debug)]
pub struct StoredTree {
    archive: Archive,
    band: Band,
    excludes: GlobSet,
}

impl StoredTree {
    /// Open the last complete version in the archive.
    pub fn open_last(archive: &Archive) -> Result<StoredTree> {
        let band = archive.last_complete_band()?;
        Ok(StoredTree {
            archive: archive.clone(),
            band,
            excludes: excludes::excludes_nothing(),
        })
    }

    /// Open a specified version.
    ///
    /// It's an error if it's not complete.
    pub fn open_version(archive: &Archive, band_id: &BandId) -> Result<StoredTree> {
        let band = Band::open(archive, band_id)?;
        if !band.is_closed()? {
            return Err(Error::BandIncomplete(band_id.clone()));
        }
        Ok(StoredTree {
            archive: archive.clone(),
            band,
            excludes: excludes::excludes_nothing(),
        })
    }

    /// Open a specified version.
    ///
    /// This function allows opening incomplete versions, which might contain only a partial copy
    /// of the source tree, or maybe nothing at all.
    pub fn open_incomplete_version(archive: &Archive, band_id: &BandId) -> Result<StoredTree> {
        let band = Band::open(archive, band_id)?;
        Ok(StoredTree {
            archive: archive.clone(),
            band,
            excludes: excludes::excludes_nothing(),
        })
    }

    pub fn with_excludes(self, excludes: GlobSet) -> StoredTree {
        StoredTree { excludes, ..self }
    }

    pub fn band(&self) -> &Band {
        &self.band
    }

    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    pub fn is_closed(&self) -> Result<bool> {
        self.band.is_closed()
    }

    pub fn validate(&self) -> Result<()> {
        let report = self.report();
        report.set_phase(format!("Check tree {}", self.band().id()));
        self.iter_entries(self.report())?
            .map(Result::unwrap)
            .filter(|e| e.kind() == Kind::File)
            .par_bridge()
            .map(|e| self.validate_one_entry(&e))
            .inspect(|e| {
                if let Err(e) = e {
                    report.problem(&e.to_string());
                }
            })
            .find_any(Result::is_err)
            .unwrap_or(Ok(()))
    }

    fn validate_one_entry(&self, e: &Entry) -> Result<()> {
        self.report().start_entry(e);
        self.open_stored_file(&e)?.validate()
    }

    /// Open a file stored within this tree.
    fn open_stored_file(&self, entry: &Entry) -> Result<StoredFile> {
        Ok(StoredFile::open(
            self.archive.block_dir().clone(),
            entry.addrs.clone(),
            self.report(),
        ))
    }

    // TODO: Perhaps add a way to open a file by name, bearing in mind this might be slow to
    // call if it reads the whole index.
}

impl ReadTree for StoredTree {
    type I = index::Iter;
    type R = ReadStoredFile;

    /// Return an iter of index entries in this stored tree.
    fn iter_entries(&self, report: &Report) -> Result<index::Iter> {
        self.band.index().iter(&self.excludes, report)
    }

    fn file_contents(&self, entry: &Entry) -> Result<Self::R> {
        Ok(self.open_stored_file(entry)?.into_read())
    }

    fn estimate_count(&self) -> Result<u64> {
        self.band.index().estimate_entry_count()
    }
}

impl HasReport for StoredTree {
    fn report(&self) -> &Report {
        self.archive.report()
    }
}

#[cfg(test)]
mod test {
    use super::super::test_fixtures::*;
    use super::super::*;

    #[test]
    pub fn open_stored_tree() {
        let af = ScratchArchive::new();
        af.store_two_versions();

        let last_band_id = af.last_band_id().unwrap();
        let st = StoredTree::open_last(&af).unwrap();

        assert_eq!(st.band().id(), last_band_id);

        let names: Vec<String> = st
            .iter_entries(&af.report())
            .unwrap()
            .map(|e| e.unwrap().apath.into())
            .collect();
        let expected = if SYMLINKS_SUPPORTED {
            vec![
                "/",
                "/hello",
                "/hello2",
                "/link",
                "/subdir",
                "/subdir/subfile",
            ]
        } else {
            vec!["/", "/hello", "/hello2", "/subdir", "/subdir/subfile"]
        };
        assert_eq!(expected, names);
    }

    #[test]
    pub fn cant_open_no_versions() {
        let af = ScratchArchive::new();
        assert!(StoredTree::open_last(&af).is_err());
    }
}
