use super::index;
use super::lexer;
use super::record;
use super::record::query;
use super::store;

use ahash::AHasher;
use crossbeam_channel::{bounded, Receiver, Sender};
use hashbrown::HashSet;
use log::debug;
use std::hash::Hasher;
use std::sync::Arc;
use std::thread::spawn;

pub struct SingleStorageBackend {
    store: store::RecordStore,
    index: index::Index,
}

impl SingleStorageBackend {
    pub fn raw_add(&mut self, line: String) {
        let result = lexer::parse_record(line.as_str());
        if result.is_err() {
            println!("{}", line);
            return;
        }

        self.add(result.unwrap());
    }
}

impl SingleStorageBackend {
    fn new() -> SingleStorageBackend {
        SingleStorageBackend {
            store: store::RecordStore::new(),
            index: index::Index::new(),
        }
    }

    fn add(&mut self, record: record::SmallRecord) -> Option<u32> {
        let tuple = self.store.add(&record);
        match tuple {
            Some(tuple) => {
                self.index.insert_record(tuple.0, &tuple.1);
                Some(tuple.0)
            }
            _ => None,
        }
    }

    fn search(&self, search_query: query::Search) -> Vec<Arc<record::RCRecord>> {
        self.store.multi_get(self.index.search(&search_query))
    }

    fn key_values_search(&self, key_values_search_query: query::KeyValuesSearch) -> Vec<Arc<str>> {
        match self.index.key_values_search(&key_values_search_query) {
            index::KeyValuesSearchResult::Ok(x) => {
                debug!("Search in normal mode (index filtering)");
                return x;
            }
            index::KeyValuesSearchResult::DirtyOk(x) => {
                debug!("Search in dirty mode (post filtering)");
                x.iter()
                    .filter_map(|id| {
                        let record = match self.store.get(*id) {
                            Some(val) => val,
                            None => return None,
                        };
                        let pair = record
                            .label_pairs
                            .iter()
                            .find(|pair| pair.key.as_ref() == key_values_search_query.key_field.as_ref());
                        match pair {
                            Some(pair) => Some(pair.val.clone()),
                            None => None,
                        }
                        // TODO return ARC instead of converting to Box
                    })
                    .collect()
            }
            index::KeyValuesSearchResult::Err(_) => Vec::new(),
        }
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
        record: record::SmallRecord,
        response_chan: Sender<Option<u32>>,
    },
    SearchRequest {
        query: query::Search,
        response_chan: Sender<Arc<record::RCRecord>>,
    },
    KeyValuesSearchRequest {
        query: query::KeyValuesSearch,
        response_chan: Sender<Arc<str>>,
    },
}

fn shard_handler(request_rcv: Receiver<BackendRequest>) {
    let mut backend = SingleStorageBackend::new();
    loop {
        match request_rcv.recv().unwrap() {
            BackendRequest::RawAddRequest { line } => {
                backend.raw_add(line);
            }
            BackendRequest::AddRequest { record, response_chan } => {
                response_chan.send(backend.add(record)).unwrap();
            }
            BackendRequest::SearchRequest { query, response_chan } => backend.search(query).into_iter().for_each(|x| {
                response_chan.send(x).unwrap();
            }),
            BackendRequest::KeyValuesSearchRequest { query, response_chan } => backend.key_values_search(query).into_iter().for_each(|x| {
                response_chan.send(x).unwrap();
            }),
        };
    }
}

pub struct ShardedStorageBackend {
    shards: Vec<Sender<BackendRequest>>,
    hasher: AHasher,
}

impl ShardedStorageBackend {
    pub fn new_with_cpus(num_cpu: u16) -> ShardedStorageBackend {
        // TODO add auto discover feature
        let mut shards: Vec<Sender<BackendRequest>> = vec![];
        for _ in 0..num_cpu {
            let (s, r) = bounded(10000);
            spawn(move || shard_handler(r));
            shards.push(s);
        }
        ShardedStorageBackend {
            shards,
            hasher: AHasher::new_with_keys(0, 0),
        }
    }

    pub fn raw_add(&self, line: String) {
        let mut hasher = self.hasher.clone();
        hasher.write(line.as_bytes());
        let hash = hasher.finish();
        self.shards[hash as usize % self.shards.len()]
            .send(BackendRequest::RawAddRequest { line })
            .unwrap();
    }

    // pub fn add(&self, record: record::Record) -> Option<u32> {
    //     let mut hasher = self.hasher.clone();
    //     hasher.write(serde_json::to_string(&record).unwrap().as_bytes());
    //     let hash = hasher.finish();
    //     let (s, r) = bounded(1);
    //     self.shards[hash as usize % self.shards.len()].send(BackendRequest::AddRequest {record: record, response_chan: s}).unwrap();
    //     r.recv().unwrap()
    // }

    pub fn search(&self, search_query: query::Search) -> Vec<Arc<record::RCRecord>> {
        let (s, r) = bounded(1000);
        (&self.shards).into_iter().for_each(|shard| {
            shard
                .send(BackendRequest::SearchRequest {
                    query: search_query.clone(),
                    response_chan: s.clone(),
                })
                .unwrap();
        });
        drop(s);
        r.into_iter().map(|f| f).collect()
    }

    pub fn key_values_search(&self, search_query: query::KeyValuesSearch) -> Vec<Arc<str>> {
        let (s, r) = bounded(1000);
        (&self.shards).into_iter().for_each(|shard| {
            shard
                .send(BackendRequest::KeyValuesSearchRequest {
                    query: search_query.clone(),
                    response_chan: s.clone(),
                })
                .unwrap();
        });
        drop(s);
        let result: HashSet<Arc<str>> = r.into_iter().map(|f| f).collect();
        result.into_iter().collect()
    }

    pub fn wait_pending_operations(&self) {
        loop {
            let empty = (&self.shards).into_iter().fold(true, |a, s| a && s.is_empty());
            if empty {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
