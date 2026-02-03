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

## Usage

```bash
# List all indices
es-cli list

# Get mapping for an index
es-cli get my-index

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
# Simple field match
es-cli kql logs 'status:error'

# AND/OR operators
es-cli kql logs 'status:error AND host:prod-*'
es-cli kql logs 'status:error OR status:warning'

# Wildcard
es-cli kql logs 'message:*timeout*'

# Range
es-cli kql logs 'response_time:>1000'

# Limit results
es-cli kql logs 'status:error' -n 50
```

## License

MIT
