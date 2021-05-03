use super::record;
use std::rc::Rc;
mod store;
mod index;

pub struct StorageBackend {
    store: store::RecordStore,
    index: index::Index
}

impl StorageBackend {
    pub fn new() -> StorageBackend {
        StorageBackend {
            store: store::RecordStore::new(),
            index: index::Index::new(),
        }
    }

    pub fn add(&mut self, record: record::Record) -> Option<u32> {
        let id = self.store.add(&record);
        match id {
            Some(id) => {
                self.index.insert_record(id, &record);
                Some(id)
            }
            _ => None
        }
    }

    pub fn search(&self, search_query: &record::SearchQuery) -> Vec<&Rc<record::RCRecord>> {
        self.store.multi_get(self.index.search(search_query))
    }

}

