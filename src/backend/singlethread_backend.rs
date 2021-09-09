use crate::index;
use crate::lexer;
use crate::record;
use crate::record::query;
use crate::store;

use log::{debug, error};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

pub struct SingleStorageBackend {
    shard_id: u16,
    store: store::RecordStore,
    index: index::Index,
}

#[derive(Serialize, Deserialize)]
pub struct SingleStorageBackendStatus {
    shard_id: u16,
    store_status: store::RecordStoreStatus,
    index_status: index::IndexStatus,
}

impl SingleStorageBackend {
    pub fn new(shard_id: u16) -> SingleStorageBackend {
        SingleStorageBackend {
            shard_id,
            store: store::RecordStore::new(),
            index: index::Index::new(),
        }
    }

    pub fn raw_add(&mut self, line: String) {
        let result = lexer::parse_record(line.as_str());
        match result {
            Ok(r) => {
                self.add(r);
            }
            Err(e) => {
                error!("‡{} (on record {})", e, line);
            }
        };
    }

    pub fn add(&mut self, record: record::SmallRecord) -> Option<u32> {
        let tuple = self.store.add(&record);
        match tuple {
            Some(tuple) => {
                self.index.insert_record(tuple.0, &tuple.1);
                Some(tuple.0)
            }
            _ => None,
        }
    }

    pub fn search(&self, search_query: query::Search) -> Vec<Arc<record::RCRecord>> {
        match search_query.is_match_all() {
            // TODO: implement dynamic limit
            true => self.store.get_all(10000),
            false => self.store.multi_get(self.index.search(&search_query)),
        }
    }

    pub fn key_values_search(&self, key_values_search_query: query::KeyValuesSearch) -> Vec<Arc<str>> {
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

    #[allow(dead_code)]
    pub fn print_status(&self) {
        self.store.print_status();
    }

    pub fn get_status(&self) -> SingleStorageBackendStatus {
        SingleStorageBackendStatus {
            shard_id: self.shard_id,
            store_status: self.store.get_status(),
            index_status: self.index.get_status(),
        }
    }
}
