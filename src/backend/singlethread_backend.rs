use crate::index;
use crate::lexer;
use crate::record;
use crate::record::query;
use crate::store;

use hashbrown::HashSet;
use log::{debug, error};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

pub trait SingleThreadBackend {
    fn new() -> Self;
    fn raw_add(&mut self, line: String);
    fn add(&mut self, record: record::SmallRecord) -> Option<u32>;
    fn search(&self, search_query: query::Search) -> Vec<Arc<record::RCRecord>>;
    fn key_values_search(&self, key_values_search_query: query::KeyValuesSearch) -> Vec<Arc<str>>;
    fn print_status(&self);
    fn get_status(&self) -> SingleStorageBackendStatus;
}

pub struct SingleStorageBackend {
    store: store::RecordStore,
    index: index::Index,
    symbol_store: HashSet<Arc<str>>,
}

#[derive(Serialize, Deserialize)]
pub struct SingleStorageBackendStatus {
    store_status: store::RecordStoreStatus,
    index_status: index::IndexStatus,
}

impl SingleStorageBackend {
    fn new_rcrecord_from(&mut self, record: &record::SmallRecord) -> record::RCRecord {
        let label_pairs = (&record.label_pairs)
            .iter()
            .map(|l| {
                let key = self.symbol_store.get_or_insert_with(l.key.as_str(), |x| Arc::from(x)).clone();
                let val = self.symbol_store.get_or_insert_with(l.val.as_str(), |x| Arc::from(x)).clone();
                record::RCLabelPair { key, val }
            })
            .collect();
        record::RCRecord::new(label_pairs)
    }
}

impl SingleThreadBackend for SingleStorageBackend {
    fn new() -> SingleStorageBackend {
        SingleStorageBackend {
            store: store::RecordStore::new(),
            index: index::Index::new(),
            symbol_store: HashSet::new(),
        }
    }

    fn raw_add(&mut self, line: String) {
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

    fn add(&mut self, record: record::SmallRecord) -> Option<u32> {
        let new_record = self.new_rcrecord_from(&record);
        let tuple = self.store.add(new_record);
        match tuple {
            Some(tuple) => {
                self.index.insert_record(tuple.0, &tuple.1);
                Some(tuple.0)
            }
            _ => None,
        }
    }

    fn search(&self, search_query: query::Search) -> Vec<Arc<record::RCRecord>> {
        match search_query.is_match_all() {
            // TODO: implement dynamic limit
            true => self.store.get_all(10000),
            false => self.store.multi_get(self.index.search(&search_query)),
        }
    }

    fn key_values_search(&self, key_values_search_query: query::KeyValuesSearch) -> Vec<Arc<str>> {
        match self.index.key_values_search(&key_values_search_query) {
            index::KeyValuesSearchResult::Ok(x) => {
                debug!("Search in normal mode (index filtering)");
                x
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
                        pair.map(|pair| pair.val.clone())
                    })
                    .collect()
            }
            index::KeyValuesSearchResult::Err(_) => Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn print_status(&self) {
        self.store.print_status();
    }

    fn get_status(&self) -> SingleStorageBackendStatus {
        SingleStorageBackendStatus {
            store_status: self.store.get_status(),
            index_status: self.index.get_status(),
        }
    }
}
