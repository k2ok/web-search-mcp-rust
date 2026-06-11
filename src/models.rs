use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub search_type: String,
    pub max_results: Option<usize>,
    pub time_range: Option<String>,
    pub region: Option<String>,
    pub safesearch: Option<String>,
    pub page: Option<usize>,
    pub backend: Option<String>,
    pub filters: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub search_type: String,
    pub total_results: usize,
    pub results: Vec<SearchResult>,
    pub error: Option<String>,
}

pub struct SearchState {
    pub blocked_until: Option<std::time::Instant>,
    pub last_request_start: Option<std::time::Instant>,
    pub last_request_end: Option<std::time::Instant>,
    pub min_wait: u64,
    pub max_wait: u64,
    pub post_wait: u64,
}
