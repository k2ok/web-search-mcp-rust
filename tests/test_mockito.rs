#[tokio::test]
async fn test_mockito_basic() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    
    let _m = server.mock("GET", "/hello")
        .with_status(200)
        .with_body("world")
        .create_async()
        .await;
    
    let client = reqwest::Client::new();
    let res = client.get(format!("{}/hello", url)).send().await.unwrap().text().await.unwrap();
    
    assert_eq!(res, "world");
}
