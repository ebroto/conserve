// Copyright 2015, 2016, 2017 Martin Pool.

//! Access a versioned tree stored in the archive.
//!
//! Through this interface you can iterate the contents and retrieve file contents.

use super::*;


#[derive(Debug)]
pub struct StoredTree {
    archive: Archive,
    band: Band,
}


impl StoredTree {
    pub fn open(archive: Archive, band_id: Option<BandId>, report: &Report) -> Result<StoredTree> {
        let band = try!(archive.open_band_or_last(&band_id, report));
        // TODO: Maybe warn if the band's incomplete, or fail unless opening is forced?
        Ok(StoredTree {
            archive: archive,
            band: band,
        })
    }

    pub fn band(&self) -> &Band {
        &self.band
    }

    /// Return an iter of index entries in this stored tree.
    pub fn index_iter(&self, report: &Report) -> Result<index::Iter> {
        self.band.index_iter(report)
    }
}


#[cfg(test)]
mod test {
    use super::super::*;
    use super::super::test_fixtures::*;

    #[test]
    pub fn open_stored_tree() {
        let af = ScratchArchive::new();
        af.store_two_versions();

        let report = Report::new();
        let a = Archive::open(af.path(), &report).unwrap();
        let last_band_id = a.last_band_id().unwrap();
        let st = StoredTree::open(a, None, &report).unwrap();

        assert_eq!(st.band().id(), last_band_id);

        let names: Vec<String> = st.index_iter(&report).unwrap().map(|e| {e.unwrap().apath}).collect();
        let expected = if SYMLINKS_SUPPORTED {
            vec!["/", "/hello", "/hello2", "/link", "/subdir", "/subdir/subfile"]
        } else {
            vec!["/", "/hello", "/hello2", "/subdir", "/subdir/subfile"]
        };
        assert_eq!(expected, names);
    }

    #[test]
    pub fn cant_open_no_versions() {
        let af = ScratchArchive::new();
        let report = Report::new();
        let a = Archive::open(af.path(), &report).unwrap();
        assert!(StoredTree::open(a, None, &report).is_err());
    }
}
