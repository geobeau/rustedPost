use crate::backend::singlethread_backend::{SingleStorageBackend, SingleThreadBackend};

pub struct TimewindowStorageBackend {
    backend: SingleStorageBackend
}


impl SingleThreadBackend for TimewindowStorageBackend {
    fn new() -> Self {
        TimewindowStorageBackend { backend: SingleStorageBackend::new() }
    }

    fn raw_add(&mut self, line: String) {
        self.backend.raw_add(line)
    }

    fn add(&mut self, record: crate::record::SmallRecord) -> Option<u32> {
        self.backend.add(record)
    }

    fn search(&self, search_query: crate::record::query::Search) -> Vec<std::sync::Arc<crate::record::RCRecord>> {
        self.backend.search(search_query)
    }

    fn key_values_search(&self, key_values_search_query: crate::record::query::KeyValuesSearch) -> Vec<std::sync::Arc<str>> {
        self.backend.key_values_search(key_values_search_query)
    }

    fn print_status(&self) {
        self.backend.print_status()
    }

    fn get_status(&self) -> super::singlethread_backend::SingleStorageBackendStatus {
        self.backend.get_status()
    }
}