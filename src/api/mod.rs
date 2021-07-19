
use warp::Filter;
use crate::{backend, lexer};
use crate::record::query;
use crate::record;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;

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

fn handle_search(search: RawAPIQuery, storage: Arc<RwLock<backend::ShardedStorageBackend>>) -> warp::reply::Json {
    let query = match lexer::parse_query(search.query.as_str()) {
        Some(x) => x,
        None => return warp::reply::json(&ErrorResponse{query: search.query, error: String::from("Parsing of query failed")}),
    };
    let data = match query {
        query::Query::Simple(x) => ResponseData::Records{data: storage.read().unwrap().search(x)},
        query::Query::KeyValues(x) => ResponseData::Values{data: storage.read().unwrap().key_values_search(x)},
    };
    let response = SuccessResponse{query: search.query, data};
    warp::reply::json(&response)
}


pub async fn serve(addr: impl Into<SocketAddr>, storage: Arc<RwLock<backend::ShardedStorageBackend>>) {
    let search = warp::post()
    .and(warp::path("search"))
    .and(warp::body::json())
    .map(move |search: RawAPIQuery| {
        handle_search(search, storage.clone())
    });

    let www_static = warp::get().and(warp::path::end()).and(warp::fs::dir("web/"));
    warp::serve(www_static.or(search)).run(addr).await;
}
