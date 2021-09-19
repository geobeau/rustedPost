use crate::record;
use crate::record::query;

use crate::telemetry::LOCAL_SHARD_LATENCY_HISTOGRAM;
use ahash::AHasher;
use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use hashbrown::HashSet;

use std::hash::Hasher;
use std::sync::Arc;
use std::thread::spawn;
use std::time::Instant;

use super::singlethread_backend::*;

#[allow(dead_code)]
enum BackendRequest {
    StatusRequest {
        response_chan: Sender<ShardedStorageBackendStatus>,
    },
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

#[derive(Serialize, Deserialize)]
pub struct ShardedStorageBackendStatus {
    shard_status: SingleStorageBackendStatus,
    shard_id: u16
}


fn shard_handler(request_rcv: Receiver<BackendRequest>, shard_id: u16) {
    let mut backend = SingleStorageBackend::new();
    let mut start;
    let mut request;
    loop {
        request = request_rcv.recv().unwrap();
        start = Instant::now();
        match request {
            BackendRequest::StatusRequest { response_chan } => {
                response_chan.send(ShardedStorageBackendStatus {shard_status: backend.get_status(), shard_id}).unwrap();
            }
            BackendRequest::RawAddRequest { line } => {
                backend.raw_add(line);
                LOCAL_SHARD_LATENCY_HISTOGRAM.raw_add.observe(start.elapsed().as_secs_f64());
            }
            BackendRequest::AddRequest { record, response_chan } => {
                response_chan.send(backend.add(record)).unwrap();
                LOCAL_SHARD_LATENCY_HISTOGRAM.add.observe(start.elapsed().as_secs_f64());
            }
            BackendRequest::SearchRequest { query, response_chan } => {
                backend.search(query).into_iter().for_each(|x| {
                    response_chan.send(x).unwrap();
                });
                LOCAL_SHARD_LATENCY_HISTOGRAM.search.observe(start.elapsed().as_secs_f64());
            }
            BackendRequest::KeyValuesSearchRequest { query, response_chan } => {
                backend.key_values_search(query).into_iter().for_each(|x| {
                    response_chan.send(x).unwrap();
                });
                LOCAL_SHARD_LATENCY_HISTOGRAM.key_values_search.observe(start.elapsed().as_secs_f64());
            }
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
        for i in 0..num_cpu {
            let (s, r) = bounded(10000);
            spawn(move || shard_handler(r, i));
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

    pub fn get_status(&self) -> Vec<ShardedStorageBackendStatus> {
        let (s, r) = bounded(self.shards.len());
        (&self.shards).into_iter().for_each(|shard| {
            shard.send(BackendRequest::StatusRequest { response_chan: s.clone() }).unwrap();
        });
        drop(s);
        r.into_iter().map(|f| f).collect()
    }

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
