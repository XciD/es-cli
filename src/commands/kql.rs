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

/// KQL accepts `and`/`or`/`not` in any case, but this command runs on
/// `query_string`, whose Lucene syntax only treats them as operators in upper
/// case. A lower-case `or` is parsed as a term to match instead, and since
/// `default_operator` is AND it has to match for the query to return anything,
/// so `a:1 or b:2` quietly returns zero hits rather than the union. Upper-case
/// the operators on the way in. Anything inside double quotes is a phrase and
/// is left alone, as is a token like `status:or` where the word is a value.
fn uppercase_boolean_operators(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    let mut word = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    fn flush(word: &mut String, out: &mut String) {
        let is_operator = ["and", "or", "not"]
            .iter()
            .any(|operator| word.eq_ignore_ascii_case(operator));
        if is_operator {
            out.push_str(&word.to_ascii_uppercase());
        } else {
            out.push_str(word);
        }
        word.clear();
    }

    for character in query.chars() {
        if escaped {
            word.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' => {
                word.push(character);
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                word.push(character);
            }
            _ if in_quotes => word.push(character),
            // The delimiters that can end a bare operator token in Lucene.
            ' ' | '\t' | '\n' | '(' | ')' => {
                flush(&mut word, &mut out);
                out.push(character);
            }
            _ => word.push(character),
        }
    }
    flush(&mut word, &mut out);
    out
}

pub async fn run(opts: KqlOptions<'_>, human: bool) -> Result<(), String> {
    let client = EsClient::new()?;
    let path = format!("/{}/_search", opts.index);

    // Build the query using query_string which respects field mappings better than
    // simple_query_string (e.g. keyword fields, wildcards, NOT operator).
    // lenient=true prevents errors on type mismatches, analyze_wildcard enables
    // wildcard expansion on analyzed fields.
    let query_clause = json!({
        "query_string": {
            "query": uppercase_boolean_operators(opts.query),
            "default_operator": "AND",
            "lenient": true,
            "analyze_wildcard": true
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

#[cfg(test)]
mod tests {
    use super::uppercase_boolean_operators;

    #[test]
    fn uppercases_bare_operators() {
        assert_eq!(
            uppercase_boolean_operators("actor.type:USER or origin:ADMIN_CONSOLE"),
            "actor.type:USER OR origin:ADMIN_CONSOLE"
        );
        assert_eq!(
            uppercase_boolean_operators("a:1 and b:2 not c:3"),
            "a:1 AND b:2 NOT c:3"
        );
        assert_eq!(uppercase_boolean_operators("a:1 And b:2"), "a:1 AND b:2");
    }

    #[test]
    fn leaves_already_valid_queries_untouched() {
        for query in [
            "actor.type:USER OR origin:NODE",
            "json.objectRef.subresource:exec",
            "",
        ] {
            assert_eq!(uppercase_boolean_operators(query), query);
        }
    }

    #[test]
    fn parentheses_delimit_operators() {
        assert_eq!(
            uppercase_boolean_operators("(a:1 or b:2) and c:3"),
            "(a:1 OR b:2) AND c:3"
        );
    }

    #[test]
    fn spares_operators_that_are_not_operators() {
        // A phrase is searched literally, so its words stay as the user typed them.
        assert_eq!(
            uppercase_boolean_operators("message:\"cat and mouse\""),
            "message:\"cat and mouse\""
        );
        // Same word as a field value, or as part of one.
        assert_eq!(uppercase_boolean_operators("status:or"), "status:or");
        assert_eq!(uppercase_boolean_operators("region:nord"), "region:nord");
        assert_eq!(
            uppercase_boolean_operators("name:or_tool and b:2"),
            "name:or_tool AND b:2"
        );
    }

    #[test]
    fn preserves_escapes_and_whitespace() {
        assert_eq!(
            uppercase_boolean_operators("path:a\\ or\\ b or c:1"),
            "path:a\\ or\\ b OR c:1"
        );
        assert_eq!(uppercase_boolean_operators("a:1\tor\nb:2"), "a:1\tOR\nb:2");
    }
}
