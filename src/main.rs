use std::vec;
use std::time::{Instant};
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;
use log::{debug, error, info, trace, warn};
use fern;
use log;

use crate::record::SearchQuery;
mod record;
mod storage;

fn display_timed_query(storage: &storage::StorageBackend, query: &record::SearchQuery) {
    let now = Instant::now();
    let records = storage.search(query);
    info!("Searching ({}): yielded {} results in {}us ({}ms) (optimized: {})", query, records.len(), now.elapsed().as_micros(), now.elapsed().as_millis(), query.query_flags.is_all());
}

fn main() {
    const FILENAME: &str = "data/dataset.txt";

    fern::Dispatch::new()
    // Perform allocation-free log formatting
    .format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}:{}] {}",
            chrono::Local::now().format("[%Y-%m-%dT%H:%M:%S]"),
            // record.target(),
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            message
        ))
    })
    // Add blanket level filter -
    .level(log::LevelFilter::Debug)
    .chain(std::io::stdout())
    .apply().unwrap();

    
    info!("Initialising backend storage");
    let mut storage = storage::StorageBackend::new();
    let now = Instant::now();
    let mut total_count = 0;
    let mut success_count = 0;

    
    info!("Loading dataset from: {}", FILENAME);
    let file = File::open(FILENAME).unwrap();


    io::BufReader::new(file).lines().for_each(|line| {
        let id = storage.add(Rc::new(serde_json::from_str(&line.unwrap()).unwrap()));
        if id.is_some() {
            success_count += 1;
        }
        total_count += 1;
    });
    info!("Loaded {} out of {} lines in {}ms", success_count, total_count, now.elapsed().as_millis());

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new( vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English"),
        record::SearchField::new_eq("extension", "pdf")],
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English"),
        record::SearchField::new_eq("extension", "epub")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English"),
        record::SearchField::new_re("extension", "(pdf|epub)")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new_with_flags(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English"),
        record::SearchField::new_re("extension", "(pdf|epub)")],
        record::QueryFlags::empty(),
    ));
    
    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_re("author_family_name", "[tT]olkien")],
    ));

    display_timed_query(&storage, &record::SearchQuery::new_with_flags(vec![
        record::SearchField::new_re("author_family_name", "[tT]olkien")],
        record::QueryFlags::empty(),
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolkien"),
        record::SearchField::new_eq("language", "English"),
        record::SearchField::new_eq("extension", "epub")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolstoy"),
        record::SearchField::new_re("title", "A[n]?na.*")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new_with_flags(vec![
        record::SearchField::new_eq("author_family_name", "Tolstoy"),
        record::SearchField::new_re("title", "A[n]?na.*")],
        record::QueryFlags::empty(),
    ));

    display_timed_query(&storage, &record::SearchQuery::new_with_flags(vec![
        record::SearchField::new_eq("author_family_name", "Tolstoy"),
        record::SearchField::new_re("title", "Anna Karénine")],
        record::QueryFlags::empty(),
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolstoy"),
        record::SearchField::new_re("title", "Anna Karénine")]
    ));

    display_timed_query(&storage, &record::SearchQuery::new(vec![
        record::SearchField::new_eq("author_family_name", "Tolstoy"),
        record::SearchField::new_eq("title", "Anna Karénine")]
    ));   

    println!("Sleeping 60s before exiting (for memory usage snapshots)");

    std::thread::sleep(std::time::Duration::from_secs(60))
}
