use std::collections::HashMap;
use std::rc::Rc;

use super::record;

pub struct RecordStore {
    id_store: Vec<Rc<record::Record>>,
    hash_store: HashMap<Rc<record::Record>, usize>,
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            id_store: Vec::new(),
            hash_store: HashMap::new()
        }
    }

    pub fn add(&mut self, record: &record::Record) -> Option<usize> {
        let r = Rc::new(record.clone());
        let result = self.hash_store.get(&r);
        match result {
            Some(record) => None,
            _ => {
                self.id_store.push(r.clone());
                let id = self.id_store.len() -1;
                self.hash_store.insert(r, id);
                Some(id)
            }
        }
        
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
