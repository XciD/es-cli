# es-cli

Minimal CLI for Elasticsearch.

## Installation

```bash
cargo install --path .
```

Or from GitHub:

```bash
cargo install --git https://github.com/XciD/es-cli
```

## Configuration

Set these environment variables:

```bash
export ELASTICSEARCH_URL="https://your-cluster.es.cloud"
export ELASTICSEARCH_API_KEY="your-api-key"
```

## Required Privileges

Most commands work with basic `read` privilege. Some commands require additional privileges:

| Command | Required Privilege |
|---------|-------------------|
| `list`, `search`, `esql`, `kql`, `count`, `tail`, `values`, `stats`, `histogram` | `read` |
| `get`, `fields`, `aliases`, `datastreams` | `read`, `view_index_metadata` |

Example role with full read access:

```json
{
  "indices": [
    {
      "names": ["*"],
      "privileges": ["read", "view_index_metadata"]
    }
  ]
}
```

## Usage

```bash
# List datastreams
es-cli datastreams
es-cli datastreams '*audit*'

# List aliases
es-cli aliases
es-cli aliases '*logs*'

# List all indices
es-cli list

# Get mapping for an index
es-cli get my-index

# List fields and types for an index
es-cli fields my-index
es-cli fields my-index -H   # Human-readable table

# Count documents
es-cli count my-index
es-cli count my-index '{"query":{"match":{"status":"error"}}}'

# Show most recent documents (sorted by @timestamp)
es-cli tail my-index
es-cli tail my-index -n 20  # Last 20 documents

# Search with query DSL
es-cli search my-index '{"query":{"match_all":{}},"size":10}'

# Execute ES|QL query
es-cli esql 'FROM my-index | LIMIT 10'

# Search with KQL/Lucene syntax (simpler than JSON DSL)
es-cli kql my-index 'status:error AND host:prod-*' -n 20
```

Output is JSON on stdout, errors on stderr. Pipe to `jq` for formatting:

```bash
es-cli list | jq '.[] | .index'
es-cli search logs '{"query":{"match_all":{}},"size":1}' | jq '.hits.hits[0]._source'
```

### Human-readable output

Use `-H` for formatted table output:

```bash
es-cli datastreams -H
es-cli aliases -H
es-cli list -H
es-cli esql 'FROM logs | LIMIT 10' -H
```

## Field Analysis

```bash
# Top values for a field (terms aggregation)
es-cli values my-index status
es-cli values my-index status -n 20        # Top 20 values
es-cli values my-index status -n 20 -H     # Human-readable table

# Statistics for a numeric field
es-cli stats my-index response_time
es-cli stats my-index response_time -H     # Human-readable output

# Document counts over time (date histogram)
es-cli histogram my-index
es-cli histogram my-index -i 1d            # Daily buckets
es-cli histogram my-index -f created_at -i 1h -H  # Custom field, hourly
```

## Examples

### Filter by field
```bash
es-cli search my-index '{"query":{"term":{"status":"error"}},"size":10}'
```

### Time range
```bash
es-cli search my-index '{"query":{"range":{"@timestamp":{"gte":"now-1h"}}},"size":10}'
```

### Aggregation
```bash
es-cli search my-index '{"size":0,"aggs":{"by_status":{"terms":{"field":"status","size":10}}}}'
```

## ES|QL Examples

```bash
# Basic query
es-cli esql 'FROM logs | LIMIT 10'

# Filter and select fields
es-cli esql 'FROM logs | WHERE status == "error" | KEEP @timestamp, message | LIMIT 10'

# Aggregation
es-cli esql 'FROM logs | STATS count = COUNT(*) BY status'

# Time range
es-cli esql 'FROM logs | WHERE @timestamp >= NOW() - 1 hour | LIMIT 10'
```

## KQL/Lucene Examples

```bash
# Basic queries
es-cli kql logs 'status:error'
es-cli kql logs 'status:error AND host:prod-*'
es-cli kql logs 'message:*timeout*'

# Sort, filter fields, and limit results
es-cli kql logs 'status:error' -n 50 --sort '-@timestamp' --fields '@timestamp,message'

# Time filters
es-cli kql logs 'status:error' --since 1h
es-cli kql logs 'status:error' --from '2024-01-01T00:00:00Z' --to '2024-01-02T00:00:00Z'

# Special characters work without escaping (uses simple_query_string)
es-cli kql audit 'owner/repo-name'
```

## License

MIT
