use crate::client::EsClient;
use crate::format::format_output;

pub async fn run(human: bool) -> Result<(), String> {
    let client = EsClient::new()?;
    let response = client.get("/_cat/indices?format=json&s=index").await?;

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
