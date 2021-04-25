use serde::{Serialize, Deserialize};
use std::{cmp::Eq};


#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    pub label_pair: Vec<LabelPair>,
}


// impl Record {
//     pub fn new(labels: Vec<LabelPair>) -> Record {
//         Record {
//             label_pair: labels,
//         }
//     }
// }

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