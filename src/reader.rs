use anyhow::{Result, anyhow};
use reqwest::Client;
use html2text;

pub async fn fetch_page(
    client: &Client,
    url: &str,
    _output_format: &str,
    include_metadata: bool,
    _include_tables: bool,
    _include_comments: bool,
    _include_images: bool,
    _deduplicate: bool,
    max_length: usize,
    timeout: u64,
) -> Result<serde_json::Value> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(timeout))
        .send()
        .await?
        .text()
        .await?;

    if response.is_empty() {
        return Err(anyhow!("Could not download content."));
    }

    let content = html2text::from_read(response.as_bytes(), 80);
    let final_content = content.trim().to_string();
    if final_content.is_empty() {
        return Err(anyhow!("No readable text found."));
    }

    let actual_length = final_content.len();
    let truncated_content = if final_content.len() > max_length {
        &final_content[..max_length]
    } else {
        &final_content
    };

    let mut response_data = serde_json::json!({
        "url": url,
        "length": actual_length,
        "content": truncated_content,
    });

    if include_metadata {
        response_data.as_object_mut().unwrap().insert("warning".to_string(), serde_json::json!("Metadata extraction not implemented in Rust version."));
    }

    Ok(response_data)
}
