use hashbrown::HashMap;
use std::rc::Rc;

use super::record;

pub struct RecordStore {
    id_store: Vec<Rc<record::Record>>,
    hash_store: HashMap<Rc<record::Record>, u32>,
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            // TODO: Guess a good capacity instead of hardcording one
            id_store: Vec::with_capacity(2_000_000),
            hash_store: HashMap::with_capacity(2_000_000)
        }
    }

    pub fn add(&mut self, record: &Rc<record::Record>) -> Option<u32> {
        let result = self.hash_store.get(record);
        match result {
            Some(_record) => None,
            _ => {
                self.id_store.push(record.clone());
                let id = (self.id_store.len() -1) as u32;
                self.hash_store.insert(record.clone(), id);
                Some(id)
            }
        }
        
    }

    pub fn get(&self, id: u32) -> Option<&Rc<record::Record>> {
        match self.id_store.get(id as usize) {
            Some(x) => Some(x),
            None => None,
        }
    }

    pub fn multi_get(&self, ids: Vec<u32>) -> Vec<&Rc<record::Record>> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }
}
