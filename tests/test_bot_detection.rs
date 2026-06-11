use reqwest::Client;
use tokio::time::{sleep, Duration};

async fn check_blocked(ua_str: &str) -> bool {
    let client = Client::builder()
        .user_agent(ua_str)
        .build()
        .unwrap();
    
    let resp = client.get("https://duckduckgo.com/lite").send().await.unwrap();
    let text = resp.text().await.unwrap();
    text.contains("Unfortunately, bots use DuckDuckGo too.")
}

#[tokio::test]
#[ignore]
async fn test_true_agent_single_request() {
    let ua_str = "web-search-mcp-rust/0.1.0 (+https://github.com/k2ok/web-search-mcp-rust)";
    assert!(!check_blocked(ua_str).await, "True agent should not be blocked on a single request");
}

#[tokio::test]
#[ignore]
async fn test_true_agent_loop_requests() {
    let ua_str = "web-search-mcp-rust/0.1.0 (+https://github.com/k2ok/web-search-mcp-rust)";
    for i in 1..=10 {
        assert!(!check_blocked(ua_str).await, "True agent should not be blocked on request {}", i);
        sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_true_agent_spam_requests() {
    let ua_str = "web-search-mcp-rust/0.1.0 (+https://github.com/k2ok/web-search-mcp-rust)";
    for i in 1..=50 {
        assert!(!check_blocked(ua_str).await, "True agent should not be blocked on request {}", i);
    }
}
