use reqwest::{Client, Response};
use std::env;

pub struct EsClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl EsClient {
    pub fn new() -> Result<Self, String> {
        let base_url = env::var("ELASTICSEARCH_URL")
            .map_err(|_| "ELASTICSEARCH_URL not set")?;
        let api_key = env::var("ELASTICSEARCH_API_KEY")
            .map_err(|_| "ELASTICSEARCH_API_KEY not set")?;

        let client = Client::new();

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Response, String> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .get(&url)
            .header("Authorization", format!("ApiKey {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn post(&self, path: &str, body: &str) -> Result<Response, String> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .post(&url)
            .header("Authorization", format!("ApiKey {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())
    }
}
