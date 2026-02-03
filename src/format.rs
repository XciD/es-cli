use serde_json::Value;

pub fn format_output(json: &str, human: bool) -> String {
    if !human {
        return json.to_string();
    }

    let value: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };

    // Detect response type and format accordingly
    if let Some(columns) = value.get("columns") {
        // ES|QL response
        format_esql(&value, columns)
    } else if let Some(hits) = value.get("hits") {
        // Search response
        format_search(hits)
    } else if value.is_array() {
        // List indices response
        format_list(&value)
    } else {
        // Pretty print JSON for other responses
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| json.to_string())
    }
}

fn format_esql(value: &Value, columns: &Value) -> String {
    let mut output = String::new();

    // Get column names
    let col_names: Vec<&str> = columns
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                .collect()
        })
        .unwrap_or_default();

    // Header
    output.push_str(&col_names.join("\t"));
    output.push('\n');
    output.push_str(&"-".repeat(col_names.len() * 20));
    output.push('\n');

    // Rows
    if let Some(values) = value.get("values").and_then(|v| v.as_array()) {
        for row in values {
            if let Some(cells) = row.as_array() {
                let formatted: Vec<String> = cells.iter().map(format_value).collect();
                output.push_str(&formatted.join("\t"));
                output.push('\n');
            }
        }
    }

    output
}

fn format_search(hits: &Value) -> String {
    let mut output = String::new();

    // Total hits
    if let Some(total) = hits.get("total") {
        let count = total
            .get("value")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let relation = total
            .get("relation")
            .and_then(|r| r.as_str())
            .unwrap_or("eq");
        let prefix = if relation == "gte" { "≥" } else { "" };
        output.push_str(&format!("Total: {}{} hits\n\n", prefix, count));
    }

    // Documents
    if let Some(docs) = hits.get("hits").and_then(|h| h.as_array()) {
        for (i, doc) in docs.iter().enumerate() {
            output.push_str(&format!("--- [{}] ", i + 1));

            if let Some(id) = doc.get("_id").and_then(|id| id.as_str()) {
                output.push_str(id);
            }
            output.push_str(" ---\n");

            if let Some(source) = doc.get("_source") {
                output.push_str(&format_source(source, 0));
            }
            output.push('\n');
        }
    }

    output
}

fn format_source(value: &Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    let mut output = String::new();

    match value {
        Value::Object(map) => {
            for (key, val) in map {
                match val {
                    Value::Object(_) => {
                        output.push_str(&format!("{}{}:\n", prefix, key));
                        output.push_str(&format_source(val, indent + 1));
                    }
                    _ => {
                        output.push_str(&format!("{}{}: {}\n", prefix, key, format_value(val)));
                    }
                }
            }
        }
        _ => {
            output.push_str(&format!("{}{}\n", prefix, format_value(value)));
        }
    }

    output
}

fn format_list(value: &Value) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{:<50} {:>12} {:>12} {:>10}\n",
        "INDEX", "DOCS", "SIZE", "STATUS"
    ));
    output.push_str(&"-".repeat(90));
    output.push('\n');

    if let Some(indices) = value.as_array() {
        for idx in indices {
            let name = idx
                .get("index")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let docs = idx
                .get("docs.count")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let size = idx
                .get("store.size")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let status = idx
                .get("health")
                .and_then(|v| v.as_str())
                .unwrap_or("-");

            output.push_str(&format!(
                "{:<50} {:>12} {:>12} {:>10}\n",
                truncate(name, 50),
                docs,
                size,
                status
            ));
        }
    }

    output
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{}", f as i64)
                } else {
                    format!("{:.2}", f)
                }
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(_) => "{...}".to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
