use crate::models::*;
use crate::search::ddg_search;
use anyhow::Result;
use reqwest::Client;
use std::sync::{Arc, Mutex};

pub async fn search_domain(client: &Client, user_agent: &crate::user_agent::UserAgent, state: &Arc<Mutex<SearchState>>, query: &str, domain: &str) -> Result<SearchResponse> {
    let enhanced_query = format!("site:{} {}", domain, query);
    
    let req = SearchRequest {
        query: enhanced_query,
        search_type: "text".to_string(),
        max_results: Some(5),
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };

    let base_url = std::env::var("DDG_BASE_URL").unwrap_or_else(|_| "https://lite.duckduckgo.com/lite/".to_string());
    ddg_search(client, user_agent, state, req, &base_url).await
}
