use hashbrown::{HashMap, HashSet};
use log::{info};
use serde::{Deserialize, Serialize};
use std::{slice::Iter, sync::Arc};

use super::record;

struct IdChunk {
    chunk: Vec::<Arc<record::RCRecord>>
}

impl IdChunk {
    fn new() -> IdChunk {
        return IdChunk{chunk: Vec::with_capacity(2_usize.pow(16)) }
    }

    fn push(&mut self, record: Arc<record::RCRecord>) -> Option<u16> {
        if self.chunk.len() >= 2_usize.pow(16) {
            None
        } else {
            self.chunk.push(record);
            Some((self.chunk.len() - 1) as u16)
        }
    }

    fn get(&self, id: u16) -> Option<Arc<record::RCRecord>> {
        match self.chunk.get(id as usize) {
            Some(x) => Some((*x).clone()),
            None => None,
        }
    }
}

struct ChunkedIdStore {
    chunk_vec: Vec<IdChunk>
}

impl ChunkedIdStore {
    fn new() -> ChunkedIdStore {
        return ChunkedIdStore{ chunk_vec: Vec::new() }
    }

    fn push(&mut self, record: Arc<record::RCRecord>) -> u32 {
        if self.chunk_vec.is_empty() {
            self.chunk_vec.push(IdChunk::new());
        }
        let lower_bucket = match self.chunk_vec.last_mut().unwrap().push(record.clone()) {
            Some(id) => id,
            None => {
                let mut new_chunk = IdChunk::new();
                let id = new_chunk.push(record).unwrap();
                self.chunk_vec.push(new_chunk);
                id
            },
        } as u32;
        let upper_bucket = ((self.chunk_vec.len() - 1) << 16) as u32;
        return upper_bucket | lower_bucket;
    }

    fn get(&self, id: u32) -> Option<Arc<record::RCRecord>> {
        let upper_bucket = ( id >> 16) as usize;
        match self.chunk_vec.get(upper_bucket) {
            Some(chunk) => chunk.get(id as u16),
            None => None
        }
    }

    fn len(&self) -> usize {
        let upper_bucket = ((self.chunk_vec.len() - 1) << 16) as u32;
        let lower_bucket = match self.chunk_vec.last() {
            Some(chunk) => chunk.chunk.len(),
            None => 0,
        } as u32;
        return (upper_bucket | lower_bucket) as usize;
    }

    fn iter(&self) -> Iter<'_, Arc<record::RCRecord>> {
        // TODO: Make it iter over all records
        return self.chunk_vec.last().unwrap().chunk.iter()
    }
}

pub struct RecordStore {
    id_store: ChunkedIdStore,
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
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            id_store: ChunkedIdStore::new(),
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
                let id = self.id_store.push(rc.clone());
                self.hash_store.insert(rc.clone(), id);
                Some((id, rc.clone()))
            }
        }
    }

    pub fn get(&self, id: u32) -> Option<Arc<record::RCRecord>> {
        self.id_store.get(id)
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
        }
    }

    pub fn multi_get(&self, ids: Vec<u32>) -> Vec<Arc<record::RCRecord>> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }

    pub fn get_all(&self, limit: usize) -> Vec<Arc<record::RCRecord>> {
        self.id_store.iter().take(limit).cloned().collect()
    }
}
