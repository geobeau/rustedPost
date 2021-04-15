use std::collections::HashSet;
use std::rc::Rc;

use super::record;

pub struct RecordStore {
    id_store: Vec<Rc<record::Record>>,
    hash_store: HashSet<Rc<record::Record>>,
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            id_store: Vec::new(),
            hash_store: HashSet::new()
        }
    }

    pub fn add(&mut self, record: &record::Record) -> usize {
        let r = Rc::new(record.clone());
        self.id_store.push(r);
        self.id_store.len() -1
    }

    pub fn get(&self, id: usize) -> Option<&Rc<record::Record>> {
        match self.id_store.get(id) {
            Some(x) => Some(x),
            None => None,
        }
    }

    pub fn multi_get(&self, ids: Vec<usize>) -> Vec<&Rc<record::Record>> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }
}
