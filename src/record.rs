use std::{collections::HashMap, vec};

#[derive(Clone)]
pub struct Record {
    pub labelPair: Vec<LabelPair>,
}

#[derive(Clone)]
pub struct LabelPair {
    pub key: String,
    pub val: String
}
