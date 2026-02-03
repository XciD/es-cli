use crate::client::EsClient;
use crate::format::format_output;
use serde_json::json;

pub async fn run(index: &str, query: &str, size: usize, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;
    let path = format!("/{}/_search", index);

    let body = json!({
        "query": {
            "query_string": {
                "query": query
            }
        },
        "size": size
    })
    .to_string();

    let response = client.post(&path, &body).await?;

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
