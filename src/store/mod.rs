use hashbrown::{HashMap, HashSet};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::record;

pub struct RecordStore {
    id_store: Vec<Arc<record::RCRecord>>,
    hash_store: HashMap<Arc<record::RCRecord>, u32>,
    symbol_store: HashSet<Arc<str>>,
}

#[derive(Serialize, Deserialize)]
pub struct RecordStoreStatus {
    symbol_store_size: usize,
    symbol_store_hashtable_capacity: usize,
    hash_store_size: usize,
    hash_store_hashtable_capacity: usize,
    id_store_size: usize,
    id_store_hashtable_capacity: usize,
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            id_store: Vec::new(),
            hash_store: HashMap::new(),
            symbol_store: HashSet::new(),
        }
    }

    fn new_rcrecord_from(&mut self, record: &record::SmallRecord) -> record::RCRecord {
        let label_pairs = (&record.label_pairs)
            .iter()
            .map(|l| {
                let key = self.symbol_store.get_or_insert_with(l.key.as_str(), |x| Arc::from(x)).clone();
                let val = self.symbol_store.get_or_insert_with(l.val.as_str(), |x| Arc::from(x)).clone();
                record::RCLabelPair { key, val }
            })
            .collect();
        record::RCRecord { label_pairs }
    }

    pub fn add(&mut self, original_record: &record::SmallRecord) -> Option<(u32, Arc<record::RCRecord>)> {
        let new_record = self.new_rcrecord_from(original_record);
        let result = self.hash_store.get(&new_record);
        match result {
            Some(_record) => None,
            _ => {
                let rc = Arc::new(new_record);
                self.id_store.push(rc.clone());
                let id = (self.id_store.len() - 1) as u32;
                self.hash_store.insert(rc.clone(), id);
                Some((id, rc.clone()))
            }
        }
    }

    pub fn get(&self, id: u32) -> Option<Arc<record::RCRecord>> {
        match self.id_store.get(id as usize) {
            Some(x) => Some((*x).clone()),
            None => None,
        }
    }

    pub fn print_status(&self) {
        info!(
            "Size of structs: symbols: {}, hashes: {}, ids: {}",
            self.symbol_store.len(),
            self.hash_store.len(),
            self.id_store.len()
        );
    }

    pub fn get_status(&self) -> RecordStoreStatus {
        RecordStoreStatus {
            symbol_store_size: self.symbol_store.len(),
            hash_store_size: self.hash_store.len(),
            id_store_size: self.id_store.len(),
            symbol_store_hashtable_capacity: self.symbol_store.capacity(),
            hash_store_hashtable_capacity: self.hash_store.capacity(),
            id_store_hashtable_capacity: self.id_store.capacity(),
        }
    }

    pub fn multi_get(&self, ids: Vec<u32>) -> Vec<Arc<record::RCRecord>> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }

    pub fn get_all(&self, limit: usize) -> Vec<Arc<record::RCRecord>> {
        self.id_store.iter().take(limit).cloned().collect()
    }
}
