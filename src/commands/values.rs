use crate::client::EsClient;

pub async fn run(index: &str, field: &str, size: usize, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_search", index);

    let query = serde_json::json!({
        "size": 0,
        "aggs": {
            "values": {
                "terms": {
                    "field": field,
                    "size": size
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
        println!("{}", format_values_human(&body));
    } else {
        println!("{}", body);
    }
    Ok(())
}

fn format_values_human(json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };

    let mut output = String::new();
    output.push_str(&format!("{:<60} {:>15}\n", "VALUE", "COUNT"));
    output.push_str(&"-".repeat(77));
    output.push('\n');

    if let Some(buckets) = value
        .get("aggregations")
        .and_then(|a| a.get("values"))
        .and_then(|v| v.get("buckets"))
        .and_then(|b| b.as_array())
    {
        for bucket in buckets {
            let key = bucket
                .get("key")
                .map(|k| {
                    if k.is_string() {
                        k.as_str().unwrap_or("-").to_string()
                    } else {
                        k.to_string()
                    }
                })
                .unwrap_or_else(|| "-".to_string());
            let count = bucket
                .get("doc_count")
                .and_then(|c| c.as_u64())
                .unwrap_or(0);

            output.push_str(&format!("{:<60} {:>15}\n", truncate(&key, 60), count));
        }

        // Show if there are more values
        if let Some(sum_other) = value
            .get("aggregations")
            .and_then(|a| a.get("values"))
            .and_then(|v| v.get("sum_other_doc_count"))
            .and_then(|s| s.as_u64())
        {
            if sum_other > 0 {
                output.push_str(&format!("\n({} other documents not shown)\n", sum_other));
            }
        }
    }

    output
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
