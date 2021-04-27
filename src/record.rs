use serde::{Serialize, Deserialize};
use std::{cmp::Eq};
use std::fmt;
use itertools::free::join;
use bitflags::bitflags;

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub label_pairs: Vec<LabelPair>,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelPair {
    pub key: Box<str>,
    pub val: Box<str>
}

impl LabelPair {
    pub fn new(key: &str, val: &str) -> LabelPair {
        LabelPair {
            key: Box::from(key),
            val: Box::from(val),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub search_fields: Vec<SearchField>,
    pub query_flags: QueryFlags,
}

impl SearchQuery {
    pub fn new(search_fields: Vec<SearchField>) -> SearchQuery {
        SearchQuery {
            search_fields: search_fields,
            query_flags: QueryFlags::DEFAULT
        }
    }
}

impl fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", join(self.search_fields.clone().into_iter().map(|f| format!("{}", f)), ", "))
    }
}

bitflags! {
    pub struct QueryFlags: u8 {
        /// Instead of doing full scans, extract a range
        const OPTIMIZE_REGEX_SEARCH = 0b00000001;
        /// NOT YET IMPLEMENTED: Don't perform all intersections if the result have been reduced enough
        const ABORT_EARLY = 0b00000010;
        const DEFAULT = Self::OPTIMIZE_REGEX_SEARCH.bits | Self::ABORT_EARLY.bits;
    }
}

#[derive(Clone, Debug)]
pub struct SearchField {
    pub key: Box<str>,
    pub val: Box<str>,
    pub op: Operation,
}

impl fmt::Display for SearchField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}\"{}\"", self.key, self.op, self.val)
    }
}

impl SearchField {
    pub fn new_eq(key: &str, val: &str) -> SearchField {
        SearchField {
            key: Box::from(key),
            val: Box::from(val),
            op: Operation::Eq,
        }
    }

    pub fn new_re(key: &str, val: &str) -> SearchField {
        SearchField {
            key: Box::from(key),
            val: Box::from(val),
            op: Operation::Re,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Operation {
    Eq,
    Re
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Eq => write!(f, "=="),
            Operation::Re => write!(f, "=~"),
        }
    }
}