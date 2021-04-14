use super::record;
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

    pub fn add(&mut self, record: &record::Record) -> usize {
        let id = self.store.add(&record);
        self.index.insert_record(id, &record);
        return id
    }

    pub fn get(&self, record: &record::Record) -> Option<record::Record> {
        Some(record)
    }

    pub fn search(&self, search_query: record::Record) -> Vec<record::Record> {
        self.store.multi_get(self.index.search(search_query))
    }

}

