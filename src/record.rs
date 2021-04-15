use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Record {
    pub label_pair: Vec<LabelPair>,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct LabelPair {
    pub key: String,
    pub val: String
}
