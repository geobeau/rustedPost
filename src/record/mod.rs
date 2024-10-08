use itertools::free::join;
use ahash::RandomState;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use smallvec::SmallVec;
use std::cmp::Eq;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::str;
use std::sync::Arc;

pub mod query;

/////////////////////////// REGULAR RECORDS ///////////////////////////
#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub label_pairs: Vec<LabelPair>,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelPair {
    pub key: Box<str>,
    pub val: Box<str>,
}

impl LabelPair {
    pub fn new(key: &str, val: &str) -> LabelPair {
        LabelPair {
            key: Box::from(key),
            val: Box::from(val),
        }
    }
}

/////////////////////////// RC RECORDS ///////////////////////////
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RCRecord {
    pub label_pairs: Vec<RCLabelPair>,
    hash_cache: u64
}

impl RCRecord {
    pub fn new(pairs: Vec<RCLabelPair>) -> RCRecord {
        let mut state = RandomState::new().build_hasher();
        pairs.hash(&mut state);
        RCRecord {
            label_pairs: pairs,
            hash_cache: state.finish(),
        }
    }
}

impl Hash for RCRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_cache);
    }
}

impl fmt::Display for RCRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", join(self.label_pairs.clone().into_iter().map(|f| format!("{}", f)), ", "))
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct RCLabelPair {
    pub key: Arc<str>,
    pub val: Arc<str>,
}

impl RCLabelPair {
    pub fn new(key: &str, val: &str) -> RCLabelPair {
        RCLabelPair {
            key: Arc::from(key),
            val: Arc::from(val),
        }
    }
}

impl fmt::Display for RCLabelPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}=={}", self.key, self.val)
    }
}

/////////////////////////// SMALL RECORDS ///////////////////////////
// Small records are like regular records except they are made to stay
// in stack
pub struct SmallRecord {
    pub label_pairs: SmallVec<[SmallLabelPair; 16]>,
}

pub struct SmallLabelPair {
    pub key: SmallString<[u8; 16]>,
    pub val: SmallString<[u8; 32]>,
}
