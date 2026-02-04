mod client;
mod commands;
mod format;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "es-cli",
    version,
    about = "Minimal CLI for Elasticsearch",
    long_about = "A minimal CLI to interact with Elasticsearch.\n\n\
                  Requires environment variables:\n  \
                  - ELASTICSEARCH_URL: Cluster URL\n  \
                  - ELASTICSEARCH_API_KEY: API key for authentication\n\n\
                  Output is JSON on stdout, errors on stderr."
)]
struct Cli {
    /// Human-readable table output instead of JSON
    #[arg(short = 'H', long, global = true)]
    human: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List index aliases (GET /_alias)
    Aliases {
        /// Optional pattern to filter aliases (supports wildcards, e.g., "*audit*")
        pattern: Option<String>,
    },

    /// Count documents in an index (GET /<index>/_count)
    Count {
        /// Index name or pattern
        index: String,

        /// Optional query DSL as JSON (e.g., '{"query":{"match":{"status":"error"}}}')
        #[arg(value_name = "JSON")]
        query: Option<String>,
    },

    /// List datastreams (GET /_data_stream)
    Datastreams {
        /// Optional pattern to filter datastreams (supports wildcards, e.g., "*audit*")
        pattern: Option<String>,
    },

    /// List fields and their types for an index (GET /<index>/_mapping)
    Fields {
        /// Index name or pattern
        index: String,
    },

    /// Show document counts over time (date histogram)
    Histogram {
        /// Index name or pattern
        index: String,

        /// Date field to aggregate (default: @timestamp)
        #[arg(short = 'f', long, default_value = "@timestamp")]
        field: String,

        /// Time interval (e.g., "1h", "1d", "5m")
        #[arg(short = 'i', long, default_value = "1h")]
        interval: String,
    },

    /// List all indices (GET /_cat/indices?format=json)
    List,

    /// Get mapping for an index (GET /<index>/_mapping)
    Get {
        /// Index name or pattern (e.g., "my-index" or "logs-*")
        index: String,
    },

    /// Search an index with a JSON query body (POST /<index>/_search)
    Search {
        /// Index name or pattern to search
        index: String,

        /// Elasticsearch query DSL as JSON string
        #[arg(value_name = "JSON")]
        query: String,
    },

    /// Execute an ES|QL query (POST /_query)
    #[command(name = "esql")]
    Esql {
        /// ES|QL query string (e.g., "FROM logs | LIMIT 10")
        query: String,
    },

    /// Search with KQL/Lucene query string syntax
    #[command(name = "kql")]
    Kql {
        /// Index name or pattern to search
        index: String,

        /// KQL/Lucene query (e.g., "status:error AND host:prod-*")
        query: String,

        /// Number of results to return
        #[arg(short = 'n', long, default_value = "10")]
        size: usize,
    },

    /// Show statistics for a numeric field (min, max, avg, sum, std_dev)
    Stats {
        /// Index name or pattern
        index: String,

        /// Numeric field name
        field: String,
    },

    /// Show most recent documents from an index (sorted by @timestamp)
    Tail {
        /// Index name or pattern
        index: String,

        /// Number of documents to show
        #[arg(short = 'n', long, default_value = "10")]
        size: usize,
    },

    /// Show top unique values for a field (terms aggregation)
    Values {
        /// Index name or pattern
        index: String,

        /// Field name to aggregate
        field: String,

        /// Number of top values to show
        #[arg(short = 'n', long, default_value = "10")]
        size: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Aliases { pattern } => {
            commands::aliases::run(pattern.as_deref(), cli.human).await
        }
        Commands::Count { index, query } => {
            commands::count::run(&index, query.as_deref(), cli.human).await
        }
        Commands::Datastreams { pattern } => {
            commands::datastreams::run(pattern.as_deref(), cli.human).await
        }
        Commands::Fields { index } => commands::fields::run(&index, cli.human).await,
        Commands::Histogram {
            index,
            field,
            interval,
        } => commands::histogram::run(&index, &field, &interval, cli.human).await,
        Commands::List => commands::list::run(cli.human).await,
        Commands::Get { index } => commands::get::run(&index, cli.human).await,
        Commands::Search { index, query } => commands::search::run(&index, &query, cli.human).await,
        Commands::Esql { query } => commands::esql::run(&query, cli.human).await,
        Commands::Kql { index, query, size } => {
            commands::kql::run(&index, &query, size, cli.human).await
        }
        Commands::Stats { index, field } => commands::stats::run(&index, &field, cli.human).await,
        Commands::Tail { index, size } => commands::tail::run(&index, size, cli.human).await,
        Commands::Values { index, field, size } => {
            commands::values::run(&index, &field, size, cli.human).await
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
