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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List => commands::list::run(cli.human).await,
        Commands::Get { index } => commands::get::run(&index, cli.human).await,
        Commands::Search { index, query } => commands::search::run(&index, &query, cli.human).await,
        Commands::Esql { query } => commands::esql::run(&query, cli.human).await,
        Commands::Kql { index, query, size } => {
            commands::kql::run(&index, &query, size, cli.human).await
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
