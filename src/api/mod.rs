
use warp::{Filter, Reply, Rejection};
use crate::{backend, lexer};
use crate::record::query;
use crate::record;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;
use prometheus::{self, Encoder};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawAPIQuery {
    pub query: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    // The query that triggered the error
    pub query: String,
    pub error: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    // The query that triggered the error
    pub query: String,
    pub data: ResponseData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponseData {
    Records {data: Vec<Arc<record::RCRecord>>},
    Values {data: Vec<Arc<str>>}
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        eprintln!("could not encode prometheus metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    Ok(res)
}

fn handle_search(search: RawAPIQuery, storage: Arc<RwLock<backend::ShardedStorageBackend>>) -> warp::reply::Json {
    let query = match lexer::parse_query(search.query.as_str()) {
        Ok(x) => x,
        Err(x) => return warp::reply::json(&ErrorResponse{query: search.query, error: x}),
    };
    let data = match query {
        query::Query::Simple(x) => ResponseData::Records{data: storage.read().unwrap().search(x)},
        query::Query::KeyValues(x) => ResponseData::Values{data: storage.read().unwrap().key_values_search(x)},
    };
    let response = SuccessResponse{query: search.query, data};
    warp::reply::json(&response)
}

fn handle_status(storage: Arc<RwLock<backend::ShardedStorageBackend>>) -> warp::reply::Json {
    let per_shard_status = storage.read().unwrap().get_status();
    warp::reply::json(&per_shard_status)
}


pub async fn serve(addr: impl Into<SocketAddr>, storage: Arc<RwLock<backend::ShardedStorageBackend>>) {
    let mut storage_clone = storage.clone();
    let search = warp::post()
    .and(warp::path("search"))
    .and(warp::body::json())
    .map(move |search: RawAPIQuery| {
        handle_search(search, storage_clone.clone())
    });

    storage_clone = storage.clone();
    let status = warp::get().and(warp::path("status"))
    .map(move || {
        handle_status(storage_clone.clone())
    });

    let prometheus = warp::get().and(warp::path("metrics")).and_then(metrics_handler);
    let www_static = warp::get().and(warp::path::end()).and(warp::fs::dir("web/"));
    warp::serve(www_static.or(search).or(prometheus).or(status)).run(addr).await;
}
