// Copyright 2017 Julian Raufelder.

//! Create GlobSet from a list of strings

use globset::{Glob, GlobSet, GlobSetBuilder};

use super::*;

pub fn from_strings<I: IntoIterator<Item = S>, S: AsRef<str>>(excludes: I) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for i in excludes {
        match Glob::new(i.as_ref()) {
            Ok(g) => builder.add(g),
            Err(e) => {
                return Err(e.into());
            }
        };
    }
    builder.build().or_else(|e| Err(e.into()))
}

pub fn excludes_nothing() -> GlobSet {
    GlobSetBuilder::new().build().unwrap()
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    pub fn simple_parse() {
        let vec = vec!["fo*", "foo", "bar*"];
        let excludes = excludes::from_strings(&vec).unwrap();
        assert_eq!(excludes.matches("foo").len(), 2);
        assert_eq!(excludes.matches("foobar").len(), 1);
        assert_eq!(excludes.matches("barBaz").len(), 1);
        assert_eq!(excludes.matches("bazBar").len(), 0);
    }

    #[test]
    pub fn path_parse() {
        let excludes = excludes::from_strings(&["fo*/bar/baz*"]).unwrap();
        assert_eq!(excludes.matches("foo/bar/baz.rs").len(), 1);
    }

    #[test]
    pub fn extendend_pattern_parse() {
        let excludes = excludes::from_strings(&["fo?", "ba[abc]", "[!a-z]"]).unwrap();
        assert_eq!(excludes.matches("foo").len(), 1);
        assert_eq!(excludes.matches("fo").len(), 0);
        assert_eq!(excludes.matches("baa").len(), 1);
        assert_eq!(excludes.matches("1").len(), 1);
        assert_eq!(excludes.matches("a").len(), 0);
    }

    #[test]
    pub fn nothing_parse() {
        let excludes = excludes::excludes_nothing();
        assert!(excludes.matches("a").is_empty());
    }
}
