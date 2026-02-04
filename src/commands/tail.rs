use crate::client::EsClient;
use crate::format::format_output;

pub async fn run(index: &str, size: usize, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_search", index);

    // Query for most recent documents, sorted by @timestamp descending
    let query = serde_json::json!({
        "size": size,
        "sort": [
            { "@timestamp": { "order": "desc", "unmapped_type": "date" } }
        ],
        "query": {
            "match_all": {}
        }
    });

    let response = client.post(&path, &query.to_string()).await?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;
    println!("{}", format_output(&body, human));
    Ok(())
}
