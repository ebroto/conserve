// Clippy warns about a redundant closure, but the closure is in the error-chain crate
// and not useful to flag here.
#![allow(unknown_lints,redundant_closure)]

use std::io;
use std::path::PathBuf;
use rustc_serialize;

error_chain! {
    foreign_links {
        io::Error, Io;
        rustc_serialize::json::DecoderError, JsonDecode;
    }

    errors {
        BlockCorrupt(block_hash: String) {
        }
        NotAnArchive(path: PathBuf) {
            display("not a Conserve archive: {:?}", path)
        }
        UnsupportedArchiveVersion(version: String) {
            display("unsupported archive version: {:?}", version)
        }
    }
}
