use mockito;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use web_search_mcp_rust::models::SearchState;
use web_search_mcp_rust::research::search_domain;
use web_search_mcp_rust::user_agent::UserAgent;
use std::env;

#[tokio::test]
async fn test_search_domain_basic() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    env::set_var("DDG_BASE_URL", url.clone());

    let html_content = r#"
        <html>
            <body>
                <div class="result">
                    <a class="result-link" href="https://docs.python.org/3/library/os.html">OS Module</a>
                    <span class="result-snippet">OS module provides portable way of using OS dependent functionality.</span>
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
    let res = search_domain(&client, &user_agent, &state, "os module", "docs.python.org").await.unwrap();

    assert_eq!(res.total_results, 1);
    assert_eq!(res.results[0].title, "OS Module");
    assert!(res.results[0].url.contains("docs.python.org"));
}
