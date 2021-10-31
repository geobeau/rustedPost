use hashbrown::HashMap;
use log::{info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

    fn iter(&self) -> ChunkedIdStoreIter {
        // TODO: Make it iter over all records
        return ChunkedIdStoreIter{pointer: 0, chunk_store: self }
    }
}

struct ChunkedIdStoreIter<'a> {
    pointer: u32,
    chunk_store: &'a ChunkedIdStore
}

impl<'a> Iterator for ChunkedIdStoreIter<'a> {
    type Item = Arc<record::RCRecord>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.pointer as usize, Some(self.chunk_store.len() as usize))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.chunk_store.get(self.pointer);
        self.pointer += 1;
        r
    }
}

pub struct RecordStore {
    id_store: ChunkedIdStore,
    hash_store: HashMap<Arc<record::RCRecord>, u32>,
}

#[derive(Serialize, Deserialize)]
pub struct RecordStoreStatus {
    hash_store_size: usize,
    hash_store_hashtable_capacity: usize,
    id_store_size: usize,
}

impl RecordStore {
    pub fn new() -> RecordStore {
        RecordStore {
            id_store: ChunkedIdStore::new(),
            hash_store: HashMap::new(),
        }
    }


    pub fn add(&mut self, original_record: record::RCRecord) -> Option<(u32, Arc<record::RCRecord>)> {
        let rc = Arc::new(original_record);
        let result = self.hash_store.get(&rc);
        match result {
            Some(_record) => None,
            _ => {
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
            "Size of structs: hashes: {}, ids: {}",
            self.hash_store.len(),
            self.id_store.len()
        );
    }

    pub fn get_status(&self) -> RecordStoreStatus {
        RecordStoreStatus {
            hash_store_size: self.hash_store.len(),
            id_store_size: self.id_store.len(),
            hash_store_hashtable_capacity: self.hash_store.capacity(),
        }
    }

    pub fn multi_get(&self, ids: Vec<u32>) -> Vec<Arc<record::RCRecord>> {
        ids.into_iter().filter_map(|id| self.get(id)).collect()
    }

    pub fn get_all(&self, limit: usize) -> Vec<Arc<record::RCRecord>> {
        self.id_store.iter().take(limit).collect()
    }
}
