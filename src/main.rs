use std::vec;
use std::time::{Instant};
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;

mod record;
mod storage;


fn display_timed_query(storage: &storage::StorageBackend, query: &record::SearchQuery) {
    let now = Instant::now();
    let records = storage.search(query);
    println!("Search 1 ({}): yielded {} results in {}us ({}ms)", query, records.len(), now.elapsed().as_micros(), now.elapsed().as_millis());
}

fn main() {
    let mut storage = storage::StorageBackend::new();
    let now = Instant::now();
    let mut total_count = 0;
    let mut success_count = 0;
    let file = File::open("data/dataset.txt").unwrap();
    io::BufReader::new(file).lines().for_each(|line| {
        let id = storage.add(Rc::new(serde_json::from_str(&line.unwrap()).unwrap()));
        if id.is_some() {
            success_count += 1;
        }
        total_count += 1;
    });
    println!("Loaded {} out of {} lines in {}ms", success_count, total_count, now.elapsed().as_millis());

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien")]
    });

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien"),
                            record::SearchField::new_eq("language", "English")]
    });

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien"),
                            record::SearchField::new_eq("language", "English"),
                            record::SearchField::new_eq("extension", "pdf")]
    });

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien"),
                            record::SearchField::new_eq("language", "English"),
                            record::SearchField::new_eq("extension", "epub")]
    });

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien"),
                            record::SearchField::new_eq("language", "English"),
                            record::SearchField::new_re("extension", "(pdf|epub)")]
    });

    display_timed_query(&storage, &record::SearchQuery{
        search_fields: vec![record::SearchField::new_eq("author_family_name", "Tolkien"),
                            record::SearchField::new_eq("language", "English"),
                            record::SearchField::new_re("title", "[sS]ilmarillion")]
    });

    println!("Sleeping 60s before exiting (for memory usage snapshots)");

    std::thread::sleep(std::time::Duration::from_secs(60))
}
