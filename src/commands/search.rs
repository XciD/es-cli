use crate::client::EsClient;
use crate::format::format_output;

pub async fn run(index: &str, query: &str, human: bool) -> Result<(), String> {
    // Validate JSON before sending
    serde_json::from_str::<serde_json::Value>(query)
        .map_err(|e| format!("Invalid JSON query: {e}"))?;

    let client = EsClient::new()?;
    let path = format!("/{}/_search", index);
    let response = client.post(&path, query).await?;

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
