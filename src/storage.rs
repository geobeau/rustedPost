use super::record;
use std::rc::Rc;
mod store;
mod index;

pub trait StorageBackend {
    fn new() -> Self;
    fn add(&mut self, record: record::Record) -> Option<u32>;
    fn search(&self, search_query: &record::SearchQuery) -> Vec<&Rc<record::RCRecord>>;
    fn print_status(&self);
}


pub struct SingleStorageBackend {
    store: store::RecordStore,
    index: index::Index
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

    fn search(&self, search_query: &record::SearchQuery) -> Vec<&Rc<record::RCRecord>> {
        self.store.multi_get(self.index.search(search_query))
    }

    fn print_status(&self) {
        self.store.print_status();
    }

}

