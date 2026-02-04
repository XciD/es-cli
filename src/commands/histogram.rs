use crate::client::EsClient;

pub async fn run(index: &str, field: &str, interval: &str, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_search", index);

    let query = serde_json::json!({
        "size": 0,
        "aggs": {
            "histogram": {
                "date_histogram": {
                    "field": field,
                    "fixed_interval": interval
                }
            }
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

    if human {
        println!("{}", format_histogram_human(&body));
    } else {
        println!("{}", body);
    }
    Ok(())
}

fn format_histogram_human(json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };

    let mut output = String::new();
    output.push_str(&format!("{:<30} {:>15} {}\n", "TIMESTAMP", "COUNT", "BAR"));
    output.push_str(&"-".repeat(80));
    output.push('\n');

    if let Some(buckets) = value
        .get("aggregations")
        .and_then(|a| a.get("histogram"))
        .and_then(|h| h.get("buckets"))
        .and_then(|b| b.as_array())
    {
        // Find max count for scaling the bar
        let max_count = buckets
            .iter()
            .filter_map(|b| b.get("doc_count").and_then(|c| c.as_u64()))
            .max()
            .unwrap_or(1);

        for bucket in buckets {
            let key_as_string = bucket
                .get("key_as_string")
                .and_then(|k| k.as_str())
                .unwrap_or("-");
            let count = bucket
                .get("doc_count")
                .and_then(|c| c.as_u64())
                .unwrap_or(0);

            // Create a simple bar chart
            let bar_width = 30;
            let bar_length = if max_count > 0 {
                (count as f64 / max_count as f64 * bar_width as f64) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_length);

            output.push_str(&format!("{:<30} {:>15} {}\n", key_as_string, count, bar));
        }
    }

    output
}
