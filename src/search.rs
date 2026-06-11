use crate::models::*;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use tokio::fs;
use tokio::time::{sleep, Duration};
use std::env;
use std::sync::{Arc, Mutex};
use rand::Rng;

struct RequestEndGuard(Arc<Mutex<SearchState>>);
impl Drop for RequestEndGuard {
    fn drop(&mut self) {
        let mut state_lock = self.0.lock().unwrap();
        state_lock.last_request_end = Some(std::time::Instant::now());
    }
}

fn get_log_dir() -> String {
    env::var("WEB_SEARCH_LOG_DIR").unwrap_or_else(|_| "logs".to_string())
}

fn is_logging_enabled() -> bool {
    env::var("WEB_SEARCH_LOGGING_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

fn get_block_duration() -> u64 {
    env::var("DDG_BLOCK_DURATION")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(305)
}

async fn log_to_file(filename: &str, content: &str) {
    if is_logging_enabled() {
        log_to_file_unconditional(filename, content).await;
    }
}

async fn log_to_file_unconditional(filename: &str, content: &str) {
    let log_dir = get_log_dir();
    let _ = fs::create_dir_all(&log_dir).await;
    let file_path = format!("{}/{}", log_dir, filename);
    let _ = fs::write(&file_path, content).await;
}

pub async fn ddg_search(client: &Client, user_agent: &crate::user_agent::UserAgent, state: &Arc<Mutex<SearchState>>, request: SearchRequest, base_url: &str) -> Result<SearchResponse> {
    if request.query.trim().is_empty() {
        return Ok(SearchResponse {
            query: request.query,
            search_type: request.search_type,
            total_results: 0,
            results: vec![],
            error: Some("Query cannot be empty".to_string()),
        });
    }

    let is_blocked = {
        let state_lock = state.lock().unwrap();
        if let Some(blocked_until) = state_lock.blocked_until {
            std::time::Instant::now() < blocked_until
        } else {
            false
        }
    };

    if is_blocked {
        let err_msg = format!("too many requests. do not call us while {} secs", get_block_duration());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        log_to_file(&format!("error_{}_{}.txt", timestamp, 0), &err_msg).await;
        return Err(anyhow::anyhow!(err_msg));
    }

    let wait_duration = {
        let mut state_lock = state.lock().unwrap();
        let now = std::time::Instant::now();
        let t_target = Duration::from_secs(rand::thread_rng().gen_range(state_lock.min_wait..=state_lock.max_wait));
        
        let wait_start = state_lock.last_request_start
            .map(|start| t_target.checked_sub(now.duration_since(start)).unwrap_or(Duration::from_secs(0)))
            .unwrap_or(Duration::from_secs(0));
            
        let wait_end = state_lock.last_request_end
            .map(|end| Duration::from_secs(state_lock.post_wait).checked_sub(now.duration_since(end)).unwrap_or(Duration::from_secs(0)))
            .unwrap_or(Duration::from_secs(0));
            
        let wait_time = std::cmp::max(wait_start, wait_end);
        
        state_lock.last_request_start = Some(now + wait_time);
        wait_time
    };
    
    if wait_duration > Duration::from_secs(0) {
        sleep(wait_duration).await;
    }

    let _guard = RequestEndGuard(state.clone());

    let mut attempt = 0;
    loop {
        let url = base_url;
        let mut params = HashMap::new();
        params.insert("q", &request.query);
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let response_result = async {
            let resp = client
                .get(url)
                .query(&params)
                .header("User-Agent", user_agent.default())
                .send()
                .await?;
            let text = resp.text().await?;
            Ok::<String, anyhow::Error>(text)
        }.await;

        let response_text = match response_result {
            Ok(text) => text,
            Err(e) => {
                log_to_file(&format!("error_{}_{}.txt", timestamp, attempt), &e.to_string()).await;
                return Err(e);
            }
        };

        if is_logging_enabled() {
            let raw_file = format!("raw_{}_{}.txt", timestamp, attempt);
            log_to_file(&raw_file, &response_text).await;
        }

        if response_text.contains("Unfortunately, bots use DuckDuckGo too.") {
            {
                let mut state_lock = state.lock().unwrap();
                state_lock.blocked_until = Some(std::time::Instant::now() + Duration::from_secs(get_block_duration()));
            }
            let err_msg = "Unfortunately, bots use DuckDuckGo too.".to_string();
            log_to_file_unconditional(&format!("error_{}_{}.txt", timestamp, attempt), &err_msg).await;
            
            let raw_file = format!("raw_{}_{}.txt", timestamp, attempt);
            log_to_file_unconditional(&raw_file, &response_text).await;
            
            return Err(anyhow::anyhow!(err_msg));
        }

        let mut results = {
            let document = Html::parse_document(&response_text);
            let result_selector = Selector::parse(".result-link").unwrap();
            let body_selector = Selector::parse(".result-snippet").unwrap();
            
            let mut res = Vec::new();
            let links: Vec<_> = document.select(&result_selector).collect();
            let snippets: Vec<_> = document.select(&body_selector).collect();

            for (link_el, snippet_el) in links.into_iter().zip(snippets.into_iter()) {
                let url = link_el.value().attr("href").unwrap_or("").to_string();
                let title = link_el.text().collect::<Vec<_>>().join(" ");
                let body = snippet_el.text().collect::<Vec<_>>().join(" ");
                
                res.push(SearchResult {
                    title,
                    url,
                    body,
                });
            }
            res
        };

        if let Some(max) = request.max_results {
            results.truncate(max);
        }

        let search_response = SearchResponse {
            query: request.query.clone(),
            search_type: request.search_type.clone(),
            total_results: results.len(),
            results,
            error: None,
        };

        if is_logging_enabled() {
            let parsed_file = format!("parsed_{}_{}.json", timestamp, attempt);
            if let Ok(parsed_json) = serde_json::to_string_pretty(&search_response) {
                log_to_file(&parsed_file, &parsed_json).await;
            }
        }

        if search_response.total_results == 0 && attempt < 1 {
            attempt += 1;
            sleep(Duration::from_secs(30)).await;
            continue;
        }

        return Ok(search_response);
    }
}
