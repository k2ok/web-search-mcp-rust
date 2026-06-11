use mockito;
use reqwest::Client;
use web_search_mcp_rust::reader::fetch_page;

#[tokio::test]
async fn test_fetch_page_basic() {
    let mut server = mockito::Server::new_async().await;
    let url = format!("{}/testpage", server.url());

    let html_content = r#"
        <html>
            <body>
                <h1>Hello World</h1>
                <p>This is a test page.</p>
            </body>
        </html>
    "#;

    let _m = server.mock("GET", "/testpage")
        .with_status(200)
        .with_body(html_content)
        .create_async()
        .await;

    let client = Client::new();
    let res = fetch_page(
        &client, 
        &url, 
        "txt", 
        false, 
        true, 
        true, 
        true, 
        true, 
        15000, 
        30
    ).await.unwrap();

    assert_eq!(res["url"], url);
    assert!(res["content"].as_str().unwrap().contains("Hello World"));
    assert!(res["content"].as_str().unwrap().contains("This is a test page."));
}

#[tokio::test]
async fn test_fetch_page_empty_response() {
    let mut server = mockito::Server::new_async().await;
    let url = format!("{}/empty", server.url());

    let _m = server.mock("GET", "/empty")
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let client = Client::new();
    let res = fetch_page(
        &client, 
        &url, 
        "txt", 
        false, 
        true, 
        true, 
        true, 
        true, 
        15000, 
        30
    ).await;

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Could not download content"));
}

#[tokio::test]
async fn test_fetch_page_no_readable_text() {
    let mut server = mockito::Server::new_async().await;
    let url = format!("{}/no-text", server.url());

    let html_content = "<html><body></body></html>";

    let _m = server.mock("GET", "/no-text")
        .with_status(200)
        .with_body(html_content)
        .create_async()
        .await;

    let client = Client::new();
    let res = fetch_page(
        &client, 
        &url, 
        "txt", 
        false, 
        true, 
        true, 
        true, 
        true, 
        15000, 
        30
    ).await;

    // Depending on how html2text handles empty body, it might return empty string
    // If it returns empty string, fetch_page returns Err
    if let Err(e) = res {
        assert!(e.to_string().contains("No readable text found"));
    } else {
        // If it succeeded, we should check if content is empty (though fetch_page should have failed)
        panic!("Should have failed with 'No readable text found'");
    }
}
