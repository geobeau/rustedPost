use super::record;
use log::{info};
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use ahash::{AHasher};
use std::hash::Hasher;
use crossbeam_channel::{Sender, Receiver, bounded};
mod store;
mod index;

pub trait StorageBackend {
    fn new() -> Self;
    fn add(&mut self, record: record::Record) -> Option<u32>;
    fn search(&self, search_query: record::SearchQuery) -> Vec<Arc<record::RCRecord>>;
    fn print_status(&self);
}


pub struct SingleStorageBackend {
    store: store::RecordStore,
    index: index::Index
}

impl SingleStorageBackend {
    pub fn raw_add(&mut self, line: String) {
        let record = serde_json::from_str(line.as_str()).unwrap();
        self.add(record);
    }
}

impl StorageBackend for SingleStorageBackend {
    fn new() -> SingleStorageBackend {
        SingleStorageBackend {
            store: store::RecordStore::new(),
            index: index::Index::new(),
        }
    }

    fn add(&mut self, record: record::Record) -> Option<u32> {
        let id = self.store.add(&record);
        match id {
            Some(id) => {
                self.index.insert_record(id, &record);
                Some(id)
            }
            _ => None
        }
    }

    fn search(&self, search_query: record::SearchQuery) -> Vec<Arc<record::RCRecord>> {
        self.store.multi_get(self.index.search(&search_query))
    }

    fn print_status(&self) {
        self.store.print_status();
    }

}

enum BackendRequest {
    RawAddRequest {
        line: String,
    },
    AddRequest {
        record: record::Record,
        response_chan: Sender<Option<u32>>,
    },
    SearchRequest {
        query: record::SearchQuery,
        response_chan: Sender<Arc<record::RCRecord>>,
    }
}


fn shard_handler(request_rcv: Receiver<BackendRequest>) {
    let mut backend = SingleStorageBackend::new();
    loop {
        match request_rcv.recv().unwrap() {
            BackendRequest::RawAddRequest {line} => {backend.raw_add(line);},
            BackendRequest::AddRequest {record, response_chan} => {response_chan.send(backend.add(record));},
            BackendRequest::SearchRequest {query, response_chan} => backend.search(query).into_iter().for_each(|x| {response_chan.send(x);})
        };
    }
}

pub struct ShardedStorageBackend {
    shards: Vec<Sender<BackendRequest>>,
    hasher: AHasher
}

impl ShardedStorageBackend {
    pub fn raw_add(&self, line: String) {
        let mut hasher = self.hasher.clone();
        hasher.write(line.as_bytes());
        let hash = hasher.finish();
        self.shards[hash as usize % self.shards.len()].send(BackendRequest::RawAddRequest {line});
    }
}

impl StorageBackend for ShardedStorageBackend {
    fn new() -> ShardedStorageBackend {
        // Randomly chosen number of cpus
        // TODO either discover or add it on the CLI
        let num_cpu = 8;
        let mut shards: Vec<Sender<BackendRequest>> = vec![];
        for _ in 0..num_cpu {
            let (s, r) = bounded(10000);
            spawn(move || shard_handler(r));
            shards.push(s);
        }
        ShardedStorageBackend {
            shards: shards,
            hasher:  AHasher::new_with_keys(0,0)
        }
    }
    
    fn add(&mut self, record: record::Record) -> Option<u32> {
        let mut hasher = self.hasher.clone();
        hasher.write(serde_json::to_string(&record).unwrap().as_bytes());
        let hash = hasher.finish();
        let (s, r) = bounded(1);
        self.shards[hash as usize % self.shards.len()].send(BackendRequest::AddRequest {record: record, response_chan: s});
        r.recv().unwrap()
    }

    fn search(&self, search_query: record::SearchQuery) -> Vec<Arc<record::RCRecord>> {
        let (s, r) = bounded(1);
        (&self.shards).into_iter().for_each(|shard| {shard.send(BackendRequest::SearchRequest {query: search_query.clone(), response_chan: s.clone()});});
        drop(s);
        r.into_iter().map(|f| f).collect()
    }

    fn print_status(&self) {
    }

}



