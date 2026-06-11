use mockito;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use web_search_mcp_rust::models::{SearchRequest, SearchState};
use web_search_mcp_rust::search::ddg_search;
use web_search_mcp_rust::user_agent::UserAgent;

#[tokio::test]
async fn test_ddg_search_basic() {
    let mut server = mockito::Server::new_async().await;
    let url = format!("{}/lite", server.url());

    let html_content = r#"
        <html>
            <body>
                <div class="result">
                    <a class="result-link" href="https://example.com">Test Result</a>
                    <span class="result-snippet">Test description</span>
                </div>
            </body>
        </html>
    "#;

    let _m = server.mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_body(html_content)
        .create_async()
        .await;

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
        query: "test query".to_string(),
        search_type: "text".to_string(),
        max_results: Some(1),
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let res = ddg_search(&client, &user_agent, &state, req, &url).await.unwrap();

    assert_eq!(res.query, "test query");
    assert_eq!(res.total_results, 1);
    assert_eq!(res.results[0].title, "Test Result");
    assert_eq!(res.results[0].url, "https://example.com");
    assert_eq!(res.results[0].body, "Test description");
}

#[tokio::test]
async fn test_ddg_search_empty_query() {
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
        query: "".to_string(),
        search_type: "text".to_string(),
        max_results: None,
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let res = ddg_search(&client, &user_agent, &state, req, "http://localhost").await.unwrap();
    assert_eq!(res.total_results, 0);
    assert!(res.error.unwrap().contains("Query cannot be empty"));
}

#[tokio::test]
async fn test_ddg_search_max_results() {
    let mut server = mockito::Server::new_async().await;
    let url = format!("{}/lite", server.url());

    let html_content = r#"
        <html>
            <body>
                <div class="result">
                    <a class="result-link" href="https://ex1.com">R1</a>
                    <span class="result-snippet">S1</span>
                </div>
                <div class="result">
                    <a class="result-link" href="https://ex2.com">R2</a>
                    <span class="result-snippet">S2</span>
                </div>
            </body>
        </html>
    "#;

    let _m = server.mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_body(html_content)
        .create_async()
        .await;

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
        max_results: Some(1),
        time_range: None,
        region: None,
        safesearch: None,
        page: None,
        backend: None,
        filters: None,
    };
    let res = ddg_search(&client, &user_agent, &state, req, &url).await.unwrap();
    assert_eq!(res.total_results, 1);
}

#[tokio::test]
async fn test_ddg_search_self_blocked() {
    let client = Client::new();
    let user_agent = UserAgent::new("test-agent".to_string());
    let state = Arc::new(Mutex::new(SearchState { 
        blocked_until: Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
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
    let res = ddg_search(&client, &user_agent, &state, req, "http://localhost").await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("too many requests"));
}
