use std::vec;
use std::time::{Instant};
use std::fs::File;
use std::io::{self, BufRead};

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
        let id = storage.add(&serde_json::from_str(&line.unwrap()).unwrap());
        if id.is_some() {
            success_count += 1;
        }
        total_count += 1;
    });
    println!("Loaded {} out of {} lines in {}ms", success_count, total_count, now.elapsed().as_millis());

    now = Instant::now();
    let mut record_search = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("author_family_name"), val: String::from("Tolkien")}]
    };
    let mut records = storage.search(record_search);
    println!("Search 1 (Tolkien): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());

    now = Instant::now();
    record_search = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("author_family_name"), val: String::from("Tolkien")},
                         record::LabelPair{key: String::from("language"), val: String::from("English")}]
    };
    records = storage.search(record_search);
    println!("Search 2 (Tolkien in English): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());

    now = Instant::now();
    record_search = record::Record{
        label_pair: vec![record::LabelPair{key: String::from("author_family_name"), val: String::from("Tolkien")},
                         record::LabelPair{key: String::from("language"), val: String::from("English")},
                         record::LabelPair{key: String::from("extension"), val: String::from("pdf")}]
    };
    records = storage.search(record_search);
    println!("Search 2 (Tolkien in English and as pdf): yielded {:?} results in {}us", records.len(), now.elapsed().as_micros());
}
