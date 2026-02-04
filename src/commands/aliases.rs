use crate::client::EsClient;
use crate::format::format_output;

pub async fn run(pattern: Option<&str>, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = match pattern {
        Some(p) => format!("/_alias/{}", p),
        None => "/_alias".to_string(),
    };

    let response = client.get(&path).await?;

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
