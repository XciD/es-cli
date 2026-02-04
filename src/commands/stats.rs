use crate::client::EsClient;

pub async fn run(index: &str, field: &str, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_search", index);

    let query = serde_json::json!({
        "size": 0,
        "aggs": {
            "stats": {
                "extended_stats": {
                    "field": field
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
        println!("{}", format_stats_human(&body));
    } else {
        println!("{}", body);
    }
    Ok(())
}

fn format_stats_human(json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };

    let mut output = String::new();

    if let Some(stats) = value.get("aggregations").and_then(|a| a.get("stats")) {
        let count = stats.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
        let min = stats.get("min").and_then(|m| m.as_f64());
        let max = stats.get("max").and_then(|m| m.as_f64());
        let avg = stats.get("avg").and_then(|a| a.as_f64());
        let sum = stats.get("sum").and_then(|s| s.as_f64());
        let std_dev = stats.get("std_deviation").and_then(|s| s.as_f64());

        output.push_str(&format!("{:<20} {}\n", "Count:", count));
        output.push_str(&format!(
            "{:<20} {}\n",
            "Min:",
            min.map(format_number).unwrap_or_else(|| "-".to_string())
        ));
        output.push_str(&format!(
            "{:<20} {}\n",
            "Max:",
            max.map(format_number).unwrap_or_else(|| "-".to_string())
        ));
        output.push_str(&format!(
            "{:<20} {}\n",
            "Average:",
            avg.map(format_number).unwrap_or_else(|| "-".to_string())
        ));
        output.push_str(&format!(
            "{:<20} {}\n",
            "Sum:",
            sum.map(format_number).unwrap_or_else(|| "-".to_string())
        ));
        output.push_str(&format!(
            "{:<20} {}\n",
            "Std Deviation:",
            std_dev
                .map(format_number)
                .unwrap_or_else(|| "-".to_string())
        ));
    }

    output
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{:.2}", n)
    }
}
