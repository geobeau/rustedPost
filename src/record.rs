#[derive(Clone, Debug)]
pub struct Record {
    pub label_pair: Vec<LabelPair>,
}

#[derive(Clone, Debug)]
pub struct LabelPair {
    pub key: String,
    pub val: String
}
