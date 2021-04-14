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

    pub fn get(&self, id: usize) -> Option<record::Record> {
        match self.store.get(id) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    pub fn multi_get(&self, ids: Vec<usize>) -> Vec<record::Record> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }
}
