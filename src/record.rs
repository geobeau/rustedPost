#[derive(Clone)]
pub struct Record {
    pub label_pair: Vec<LabelPair>,
}

#[derive(Clone)]
pub struct LabelPair {
    pub key: String,
    pub val: String
}
