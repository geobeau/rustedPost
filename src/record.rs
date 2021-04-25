use serde::{Serialize, Deserialize};
use std::{cmp::Eq};


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

pub struct SearchQuery {
    pub search_fields: Vec<SearchField>,
}

pub struct SearchField {
    pub key: Box<str>,
    pub val: Box<str>,
    pub op: Operation,
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


pub enum Operation {
    Eq,
    Re
}