use mimalloc::MiMalloc;
use std::vec;
use std::time::{Instant};
use std::fs::File;
use std::io::{self, BufRead};
use log::{info};
use fern;
use log;
use std::sync::{Arc, RwLock};
use warp::Filter;
use rusted_post::record;
use rusted_post::record::query;
use rusted_post::backend;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn display_timed_query(backend: &Arc<RwLock<backend::ShardedStorageBackend>>, query: query::Search) {
    let now = Instant::now();
    let records = backend.read().unwrap().search(query.clone());
    info!("Searching ({}): yielded {} results in {}us ({}ms) (optimized: {})", &query, records.len(), now.elapsed().as_micros(), now.elapsed().as_millis(), query.query_flags.contains(record::QueryFlags::OPTIMIZE_REGEX_SEARCH));
}

fn display_timed_key_query(backend: &Arc<RwLock<storage::ShardedStorageBackend>>, query: record::KeyValuesSearch) {
    let now = Instant::now();
    let records = backend.read().unwrap().key_values_search(query.clone());
    info!("Searching ({}): yielded {} results in {}us ({}ms) (optimized: {})", &query, records.len(), now.elapsed().as_micros(), now.elapsed().as_millis(), query.query_flags.contains(record::QueryFlags::OPTIMIZE_REGEX_SEARCH));
    records.iter().for_each(|record|info!("Found: {}", record))
}

#[tokio::main]
async fn main() {
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
    .level(log::LevelFilter::Info)
    .chain(std::io::stdout())
    .apply().unwrap();

    
    info!("Initialising backend storage");
    let storage = Arc::new(RwLock::new(backend::ShardedStorageBackend::new()));
    let now = Instant::now();
    let mut total_count = 0;
    let mut success_count = 0; 

    
    info!("Loading dataset from: {}", FILENAME);
    let file = File::open(FILENAME).unwrap();

    let storage_guard = storage.read().unwrap();
    io::BufReader::new(file).lines().for_each(|line| {
        storage_guard.raw_add(line.unwrap());
        // Multithread system is fire and forget
        success_count += 1;
        total_count += 1;
    });
    storage_guard.wait_pending_operations();
    drop(storage_guard);
    info!("Loaded {} out of {} lines in {}ms ({}us per record)", success_count, total_count, now.elapsed().as_millis(), ((now.elapsed().as_millis() as f64 / total_count as f64) * 1000_f64) as u32 );


    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolkien")]
    ));

    display_timed_query(&storage, query::Search::new( vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English")]
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English"),
        query::Field::new_eq("extension", "pdf")],
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English"),
        query::Field::new_eq("extension", "epub")]
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English"),
        query::Field::new_re("extension", "(pdf|epub)")]
    ));

    display_timed_query(&storage, query::Search::new_with_flags(vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English"),
        query::Field::new_re("extension", "(pdf|epub)")],
        query::SearchFlags::empty(),
    ));
    
    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_re("author_family_name", "[tT]olkien")],
    ));

    display_timed_query(&storage, query::Search::new_with_flags(vec![
        query::Field::new_re("author_family_name", "[tT]olkien")],
        query::SearchFlags::empty(),
    ));

    // display_timed_key_query(&storage, record::KeyValuesSearch::new(vec![
    //     record::SearchField::new_re("author_family_name", "[tT]olkien"),
    //     record::SearchField::new_eq("language", "English")],
    //     "title",
    // ));

    display_timed_key_query(&storage, record::KeyValuesSearch::new(vec![
        record::SearchField::new_eq("language", "English")],
        "extension",
    ));

    display_timed_key_query(&storage, record::KeyValuesSearch::new_with_flags(vec![
        record::SearchField::new_eq("language", "English")],
        "extension",
        record::QueryFlags::ABORT_EARLY | record::QueryFlags::OPTIMIZE_REGEX_SEARCH,
    ));

    display_timed_key_query(&storage, record::KeyValuesSearch::new(vec![
        record::SearchField::new_eq("language", "Breton")],
        "title",
    ));

    display_timed_key_query(&storage, record::KeyValuesSearch::new_with_flags(vec![
        record::SearchField::new_eq("language", "Breton")],
        "title",
        record::QueryFlags::ABORT_EARLY | record::QueryFlags::OPTIMIZE_REGEX_SEARCH,
    ));


    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolkien"),
        query::Field::new_eq("language", "English"),
        query::Field::new_eq("extension", "epub")]
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolstoy"),
        query::Field::new_re("title", "A[n]?na.*")]
    ));

    display_timed_query(&storage, query::Search::new_with_flags(vec![
        query::Field::new_eq("author_family_name", "Tolstoy"),
        query::Field::new_re("title", "A[n]?na.*")],
        query::SearchFlags::empty(),
    ));

    display_timed_query(&storage, query::Search::new_with_flags(vec![
        query::Field::new_eq("author_family_name", "Tolstoy"),
        query::Field::new_re("title", "Anna Karénine")],
        query::SearchFlags::empty(),
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolstoy"),
        query::Field::new_re("title", "Anna Karénine")]
    ));

    display_timed_query(&storage, query::Search::new(vec![
        query::Field::new_eq("author_family_name", "Tolstoy"),
        query::Field::new_eq("title", "Anna Karénine")]
    ));
    
    // storage.print_status();

    let search =  warp::post()
        .and(warp::path("search"))
        .and(warp::body::json())
        .map(move |search: Vec<query::Field>| {
            let search_query = query::Search::new(search);
            display_timed_query(&storage, search_query);
            warp::reply::reply()
        });

    warp::serve(search)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
