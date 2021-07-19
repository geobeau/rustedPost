use clap::{App, Arg};
use fern;
use log;
use log::{debug, error, info};
use mimalloc::MiMalloc;
use rusted_post::{backend};
use rusted_post::record::query;
use rusted_post::api;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::vec;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn display_timed_query(backend: &Arc<RwLock<backend::ShardedStorageBackend>>, query: query::Search) {
    let now = Instant::now();
    let records = backend.read().unwrap().search(query.clone());
    info!(
        "Searching ({}): yielded {} results in {}us ({}ms) (optimized: {})",
        &query,
        records.len(),
        now.elapsed().as_micros(),
        now.elapsed().as_millis(),
        query.query_flags.contains(query::SearchFlags::OPTIMIZE_REGEX_SEARCH)
    );
}

fn display_timed_key_query(backend: &Arc<RwLock<backend::ShardedStorageBackend>>, query: query::KeyValuesSearch) {
    let now = Instant::now();
    let records = backend.read().unwrap().key_values_search(query.clone());
    info!(
        "Searching ({}): yielded {} results in {}us ({}ms) (optimized: {})",
        &query,
        records.len(),
        now.elapsed().as_micros(),
        now.elapsed().as_millis(),
        query.query_flags.contains(query::SearchFlags::OPTIMIZE_REGEX_SEARCH)
    );
    records.iter().for_each(|record| debug!("Found: {}", record))
}

fn load_data_from_file(backend: &Arc<RwLock<backend::ShardedStorageBackend>>, filename: &str) {
    info!("Loading dataset from: {}", filename);
    let file = File::open(filename).unwrap();

    let mut total_count = 0;
    let mut success_count = 0;
    let now = Instant::now();

    let storage_guard = backend.read().unwrap();
    io::BufReader::new(file).lines().for_each(|line| {
        storage_guard.raw_add(line.unwrap());
        // Multithread system is fire and forget
        success_count += 1;
        total_count += 1;
    });
    storage_guard.wait_pending_operations();
    info!(
        "Loaded {} out of {} lines in {}ms ({}us per record)",
        success_count,
        total_count,
        now.elapsed().as_millis(),
        ((now.elapsed().as_millis() as f64 / total_count as f64) * 1000_f64) as u32
    );
}


#[tokio::main]
async fn main() {
    ////////////// CLI INITIALIZATION //////////////
    let matches = App::new("Rusted Post")
        .version("1.0")
        .author("G. Beausire <geobeau@gmail.com>")
        .about("Tiny in-memory database made to search accross milions of records very quickly")
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("Number of threads")
                .about("Number of threads to use for shards")
                .default_value("4")
                .takes_value(true),
        )
        .arg(
            Arg::new("skip_startup_load")
                .long("skip-startup-load")
                .about("Skip the loading of data at startup"),
        )
        .arg(
            Arg::new("file_to_load")
                .short('d')
                .long("load-from-file")
                .value_name("Path to file")
                .about("Path of the file to load data at startup")
                .takes_value(true)
                .default_value("data/dataset_custom.txt"),
        )
        .arg(
            Arg::new("log-level")
                .short('v')
                .long("log-level")
                .takes_value(true)
                .about("Change the verbosity (debug, info, err, warn)")
                .default_value("info"),
        )
        .get_matches();

    ////////////// LOG INITIALIZATION //////////////
    let level = match matches.value_of("log-level").unwrap() {
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "err" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        _ => {
            println!("Log level didn't match a known level (debug, info, err, warn)");
            return;
        }
    };

    fern::Dispatch::new()
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
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    ////////////// BACKEND INITIALIZATION //////////////
    info!("Initialising backend storage");

    let threads = match matches.value_of("threads").unwrap().parse() {
        Ok(x) => x,
        Err(err) => {
            error!("Error while getting thread count: {}", err);
            return;
        }
    };

    let storage = Arc::new(RwLock::new(backend::ShardedStorageBackend::new_with_cpus(threads)));

    ////////////// DATA LOADING AND EXAMPLE QUERIES //////////////
    if !matches.is_present("skip_startup_load") {
        load_data_from_file(&storage, matches.value_of("file_to_load").unwrap());

        display_timed_query(&storage, query::Search::new(vec![query::Field::new_eq("author_family_name", "Tolkien")]));

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolkien"),
                query::Field::new_eq("language", "English"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolkien"),
                query::Field::new_eq("language", "English"),
                query::Field::new_eq("extension", "pdf"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolkien"),
                query::Field::new_eq("language", "English"),
                query::Field::new_eq("extension", "epub"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolkien"),
                query::Field::new_eq("language", "English"),
                query::Field::new_re("extension", "(pdf|epub)"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new_with_flags(
                vec![
                    query::Field::new_eq("author_family_name", "Tolkien"),
                    query::Field::new_eq("language", "English"),
                    query::Field::new_re("extension", "(pdf|epub)"),
                ],
                query::SearchFlags::empty(),
            ),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![query::Field::new_re("author_family_name", "[tT]olkien")]),
        );

        display_timed_query(
            &storage,
            query::Search::new_with_flags(
                vec![query::Field::new_re("author_family_name", "[tT]olkien")],
                query::SearchFlags::empty(),
            ),
        );

        display_timed_key_query(
            &storage,
            query::KeyValuesSearch::new(vec![query::Field::new_eq("language", "English")], "extension"),
        );

        display_timed_key_query(
            &storage,
            query::KeyValuesSearch::new_with_flags(
                vec![query::Field::new_eq("language", "English")],
                "extension",
                query::SearchFlags::ABORT_EARLY | query::SearchFlags::OPTIMIZE_REGEX_SEARCH,
            ),
        );

        display_timed_key_query(
            &storage,
            query::KeyValuesSearch::new(vec![query::Field::new_eq("language", "Breton")], "title"),
        );

        display_timed_key_query(
            &storage,
            query::KeyValuesSearch::new_with_flags(
                vec![query::Field::new_eq("language", "Breton")],
                "title",
                query::SearchFlags::ABORT_EARLY | query::SearchFlags::OPTIMIZE_REGEX_SEARCH,
            ),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolkien"),
                query::Field::new_eq("language", "English"),
                query::Field::new_eq("extension", "epub"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolstoy"),
                query::Field::new_re("title", "A[n]?na.*"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new_with_flags(
                vec![
                    query::Field::new_eq("author_family_name", "Tolstoy"),
                    query::Field::new_re("title", "A[n]?na.*"),
                ],
                query::SearchFlags::empty(),
            ),
        );

        display_timed_query(
            &storage,
            query::Search::new_with_flags(
                vec![
                    query::Field::new_eq("author_family_name", "Tolstoy"),
                    query::Field::new_re("title", "Anna Karénine"),
                ],
                query::SearchFlags::empty(),
            ),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolstoy"),
                query::Field::new_re("title", "Anna Karénine"),
            ]),
        );

        display_timed_query(
            &storage,
            query::Search::new(vec![
                query::Field::new_eq("author_family_name", "Tolstoy"),
                query::Field::new_eq("title", "Anna Karénine"),
            ]),
        );
    }

    // storage.print_status();
    api::serve(([0, 0, 0, 0], 8080), storage).await;
}
