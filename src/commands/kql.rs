use crate::client::EsClient;
use crate::format::format_output;
use serde_json::json;

/// Options for KQL queries
pub struct KqlOptions<'a> {
    pub index: &'a str,
    pub query: &'a str,
    pub size: usize,
    pub sort: Option<&'a str>,
    pub fields: Option<&'a str>,
    pub since: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
    pub timestamp_field: &'a str,
}

pub async fn run(opts: KqlOptions<'_>, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;
    let path = format!("/{}/_search", opts.index);

    // Build the query - use simple_query_string to avoid escaping issues with special chars like /
    let query_clause = json!({
        "simple_query_string": {
            "query": opts.query,
            "default_operator": "AND"
        }
    });

    // Add time range filter if any time options are specified
    let has_time_filter = opts.since.is_some() || opts.from.is_some() || opts.to.is_some();

    let final_query = if has_time_filter {
        let mut range = json!({});

        if let Some(since) = opts.since {
            range[opts.timestamp_field]["gte"] = json!(format!("now-{}", since));
        }
        if let Some(from) = opts.from {
            range[opts.timestamp_field]["gte"] = json!(from);
        }
        if let Some(to) = opts.to {
            range[opts.timestamp_field]["lte"] = json!(to);
        }

        json!({
            "bool": {
                "must": [query_clause],
                "filter": [{ "range": range }]
            }
        })
    } else {
        query_clause
    };

    // Build the request body
    let mut body = json!({
        "query": final_query,
        "size": opts.size
    });

    // Add sort if specified
    if let Some(sort) = opts.sort {
        let (field, order) = if let Some(stripped) = sort.strip_prefix('-') {
            (stripped, "desc")
        } else if let Some(stripped) = sort.strip_prefix('+') {
            (stripped, "asc")
        } else {
            (sort, "desc") // default to desc for most common use case (recent first)
        };
        body["sort"] = json!([{ field: order }]);
    }

    // Add _source filtering if fields specified
    if let Some(fields) = opts.fields {
        let field_list: Vec<&str> = fields.split(',').map(|s| s.trim()).collect();
        body["_source"] = json!(field_list);
    }

    let response = client.post(&path, &body.to_string()).await?;

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
