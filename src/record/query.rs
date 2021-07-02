use bitflags::bitflags;
use itertools::free::join;
use serde::{Deserialize, Serialize};
use std::cmp::Eq;
use std::fmt;
use std::str;

bitflags! {
    pub struct SearchFlags: u8 {
        /// Instead of doing full scans, extract a range
        const OPTIMIZE_REGEX_SEARCH = 0b00000001;
        /// NOT YET IMPLEMENTED: Don't perform all intersections if the result have been reduced enough
        const ABORT_EARLY = 0b00000010;
        const DEFAULT = Self::OPTIMIZE_REGEX_SEARCH.bits | Self::ABORT_EARLY.bits;
    }
}

pub enum Query {
    Simple(Search),
    KeyValues(KeyValuesSearch)
}

#[derive(Clone, Debug)]
pub struct Search {
    pub search_fields: Vec<Field>,
    pub query_flags: SearchFlags,
}

impl Search {
    pub fn new(search_fields: Vec<Field>) -> Search {
        Search {
            search_fields,
            query_flags: SearchFlags::DEFAULT,
        }
    }
    pub fn new_with_flags(search_fields: Vec<Field>, flags: SearchFlags) -> Search {
        Search {
            search_fields,
            query_flags: flags,
        }
    }
}

impl fmt::Display for Search {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}", join(self.search_fields.clone().into_iter().map(|f| format!("{}", f)), ", "))
    }
}

#[derive(Clone, Debug)]
pub struct KeyValuesSearch {
    pub search_fields: Vec<Field>,
    pub key_field: Box<str>,
    pub query_flags: SearchFlags,
}

impl KeyValuesSearch {
    pub fn new(search_fields: Vec<Field>, key: &str) -> KeyValuesSearch {
        KeyValuesSearch {
            search_fields,
            key_field: Box::from(key),
            query_flags: SearchFlags::DEFAULT,
        }
    }

    pub fn new_with_flags(search_fields: Vec<Field>, key: &str, new_with_flags: SearchFlags) -> KeyValuesSearch {
        KeyValuesSearch {
            search_fields,
            key_field: Box::from(key),
            query_flags: new_with_flags,
        }
    }

    pub fn to_search_query(&self) -> Search {
        Search {
            search_fields: self.search_fields.clone(),
            query_flags: SearchFlags::DEFAULT,
        }
    }
}

impl fmt::Display for KeyValuesSearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "label_values({{{}}},{})",
            join(self.search_fields.clone().into_iter().map(|f| format!("{}", f)), ", "),
            self.key_field
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub key: Box<str>,
    pub val: Box<str>,
    pub op: Operation,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}\"{}\"", self.key, self.op, self.val)
    }
}

impl Field {
    pub fn new_eq(key: &str, val: &str) -> Field {
        Field {
            key: Box::from(key),
            val: Box::from(val),
            op: Operation::Eq,
        }
    }

    pub fn new_re(key: &str, val: &str) -> Field {
        Field {
            key: Box::from(key),
            val: Box::from(val),
            op: Operation::Re,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Operation {
    Eq,
    Re,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Eq => write!(f, "=="),
            Operation::Re => write!(f, "=~"),
        }
    }
}
