use crate::client::EsClient;
use crate::format::format_output;

pub async fn run(index: &str, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;

    let path = format!("/{}/_mapping", index);
    let response = client.get(&path).await?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }

    let body = response.text().await.map_err(|e| e.to_string())?;

    // For human output, we extract and flatten the fields
    if human {
        println!("{}", format_fields_human(&body));
    } else {
        println!("{}", format_output(&body, false));
    }
    Ok(())
}

fn format_fields_human(json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };

    let mut output = String::new();
    output.push_str(&format!("{:<60} {:<20}\n", "FIELD", "TYPE"));
    output.push_str(&"-".repeat(82));
    output.push('\n');

    let mut fields: Vec<(String, String)> = Vec::new();

    // Iterate over indices in the response
    if let Some(obj) = value.as_object() {
        for (_index_name, index_data) in obj {
            if let Some(mappings) = index_data.get("mappings") {
                if let Some(properties) = mappings.get("properties") {
                    collect_fields(properties, "", &mut fields);
                }
            }
        }
    }

    fields.sort_by(|a, b| a.0.cmp(&b.0));
    fields.dedup();

    for (field, field_type) in fields {
        output.push_str(&format!(
            "{:<60} {:<20}\n",
            truncate(&field, 60),
            field_type
        ));
    }

    output
}

fn collect_fields(
    properties: &serde_json::Value,
    prefix: &str,
    fields: &mut Vec<(String, String)>,
) {
    if let Some(obj) = properties.as_object() {
        for (name, field_data) in obj {
            let full_name = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}.{}", prefix, name)
            };

            if let Some(field_type) = field_data.get("type").and_then(|t| t.as_str()) {
                fields.push((full_name.clone(), field_type.to_string()));
            }

            // Handle nested properties
            if let Some(nested_props) = field_data.get("properties") {
                collect_fields(nested_props, &full_name, fields);
            }

            // Handle multi-fields (e.g., text with keyword sub-field)
            if let Some(multi_fields) = field_data.get("fields") {
                if let Some(mf_obj) = multi_fields.as_object() {
                    for (mf_name, mf_data) in mf_obj {
                        let mf_full_name = format!("{}.{}", full_name, mf_name);
                        if let Some(mf_type) = mf_data.get("type").and_then(|t| t.as_str()) {
                            fields.push((mf_full_name, mf_type.to_string()));
                        }
                    }
                }
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
