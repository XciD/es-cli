use crate::client::EsClient;

pub async fn run(index: &str, query: Option<&str>, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_count", index);

    let response = match query {
        Some(q) => {
            // Validate JSON
            serde_json::from_str::<serde_json::Value>(q)
                .map_err(|e| format!("Invalid JSON query: {}", e))?;
            client.post(&path, q).await?
        }
        None => client.post(&path, r#"{"query":{"match_all":{}}}"#).await?,
    };

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;

    if human {
        // Parse and show just the count
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(count) = value.get("count").and_then(|c| c.as_u64()) {
                println!("{}", count);
                return Ok(());
            }
        }
    }

    println!("{}", body);
    Ok(())
}
