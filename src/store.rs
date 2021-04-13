use super::record;

pub struct RecordStore {
    store: Vec<record::Record>
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {store: Vec::new()}
    }

    pub fn add(&mut self, record: &record::Record) -> usize {
        self.store.push(record.clone());
        self.store.len() -1
    }

    pub fn get(&mut self, index: usize) -> Option<record::Record> {
        match self.store.get(index) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}
