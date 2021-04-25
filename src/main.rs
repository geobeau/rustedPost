use std::vec;
use std::time::{Instant};
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;

mod record;
mod storage;

// https://doc.rust-lang.org/std/default/trait.Default.html

fn main() {
    let mut storage = storage::StorageBackend::new();

    let mut now = Instant::now();
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

    now = Instant::now();
    let mut record_search = record::SearchQuery{
        search_fields: vec![record::SearchField::newEq("author_family_name", "Tolkien")]
    };
    let mut records = storage.search(record_search);
    println!("Search 1 (Tolkien): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());

    now = Instant::now();
    record_search = record::SearchQuery{
        search_fields: vec![record::SearchField::newEq("author_family_name", "Tolkien"),
                            record::SearchField::newEq("language", "English")]
    };
    records = storage.search(record_search);
    println!("Search 2 (Tolkien in English): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());

    now = Instant::now();
    record_search = record::SearchQuery{
        search_fields: vec![record::SearchField::newEq("author_family_name", "Tolkien"),
                            record::SearchField::newEq("language", "English"),
                            record::SearchField::newEq("extension", "pdf")]
    };
    records = storage.search(record_search);
    println!("Search 2 (Tolkien in English and as pdf): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());
    println!("Sleeping 60s before exiting (for memory usage snapshots)");

    std::thread::sleep(std::time::Duration::from_secs(60))
}
