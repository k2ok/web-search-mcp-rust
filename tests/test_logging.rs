use mockito;
use reqwest::Client;
use std::sync::{Arc, Mutex, OnceLock};
use std::env;
use std::fs;
use web_search_mcp_rust::models::{SearchRequest, SearchState};
use web_search_mcp_rust::search::ddg_search;
use web_search_mcp_rust::user_agent::UserAgent;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

async fn setup_test_env(log_dir: &str, logging_enabled: bool) {
    env::set_var("WEB_SEARCH_LOG_DIR", log_dir);
    env::set_var("WEB_SEARCH_LOGGING_ENABLED", if logging_enabled { "true" } else { "false" });
    let _ = fs::remove_dir_all(log_dir);
    let _ = fs::create_dir_all(log_dir);
}

#[tokio::test]
async fn test_logging_disabled_normal_no_log() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let log_dir = "test_logs_disabled_normal";
    setup_test_env(log_dir, false).await;
    
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let html_content = "<html><body><div>Test</div></body></html>";
    let _m = server.mock("GET", mockito::Matcher::Any).with_status(200).with_body(html_content).create_async().await;

    let client = Client::new();
    let user_agent = UserAgent::new("test-agent".to_string());
    let state = Arc::new(Mutex::new(SearchState { 
        blocked_until: None,
        last_request_start: None,
        last_request_end: None,
        min_wait: 1,
        max_wait: 5,
        post_wait: 1,
    }));
    let req = SearchRequest {
        query: "test".to_string(),
        search_type: "text".to_string(),
        max_results: None,
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let _ = ddg_search(&client, &user_agent, &state, req, &url).await;

    let paths = fs::read_dir(log_dir).unwrap();
    assert_eq!(paths.count(), 0, "No logs should be created when logging is disabled for normal response");
    let _ = fs::remove_dir_all(log_dir);
}

#[tokio::test]
async fn test_logging_disabled_block_logs() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let log_dir = "test_logs_disabled_block";
    setup_test_env(log_dir, false).await;
    
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let html_content = "Unfortunately, bots use DuckDuckGo too.";
    let _m = server.mock("GET", mockito::Matcher::Any).with_status(200).with_body(html_content).create_async().await;

    let client = Client::new();
    let user_agent = UserAgent::new("test-agent".to_string());
    let state = Arc::new(Mutex::new(SearchState { 
        blocked_until: None,
        last_request_start: None,
        last_request_end: None,
        min_wait: 1,
        max_wait: 5,
        post_wait: 1,
    }));
    let req = SearchRequest {
        query: "test".to_string(),
        search_type: "text".to_string(),
        max_results: None,
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let _ = ddg_search(&client, &user_agent, &state, req, &url).await;

    let paths = fs::read_dir(log_dir).unwrap();
    let count = paths.count();
    assert!(count >= 2, "At least 2 logs (error and raw) should be created on block even if logging is disabled. Found: {}", count);
    let _ = fs::remove_dir_all(log_dir);
}

#[tokio::test]
async fn test_logging_enabled_all_logs() {
    let _lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let log_dir = "test_logs_enabled";
    setup_test_env(log_dir, true).await;
    
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let html_content = "<html><body><div>Test</div></body></html>";
    let _m = server.mock("GET", mockito::Matcher::Any).with_status(200).with_body(html_content).create_async().await;

    let client = Client::new();
    let user_agent = UserAgent::new("test-agent".to_string());
    let state = Arc::new(Mutex::new(SearchState { 
        blocked_until: None,
        last_request_start: None,
        last_request_end: None,
        min_wait: 1,
        max_wait: 5,
        post_wait: 1,
    }));
    let req = SearchRequest {
        query: "test".to_string(),
        search_type: "text".to_string(),
        max_results: None,
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let _ = ddg_search(&client, &user_agent, &state, req, &url).await;

    let paths = fs::read_dir(log_dir).unwrap();
    assert!(paths.count() >= 2, "Logs (raw and parsed) should be created when logging is enabled");
    let _ = fs::remove_dir_all(log_dir);
}
