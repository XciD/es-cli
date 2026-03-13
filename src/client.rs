use reqwest::{Client, RequestBuilder, Response};
use std::env;

enum Auth {
    ApiKey(String),
    Basic { username: String, password: String },
}

pub struct EsClient {
    client: Client,
    base_url: String,
    auth: Auth,
}

impl EsClient {
    pub fn new() -> Result<Self, String> {
        let base_url = env::var("ELASTICSEARCH_URL").map_err(|_| "ELASTICSEARCH_URL not set")?;

        let auth = if let Ok(key) = env::var("ELASTIC_API_KEY").or_else(|_| env::var("ELASTICSEARCH_API_KEY")) {
            Auth::ApiKey(key)
        } else if let (Ok(username), Ok(password)) = (env::var("ELASTIC_USERNAME"), env::var("ELASTIC_PASSWORD")) {
            Auth::Basic { username, password }
        } else {
            return Err(
                "No authentication configured. Set ELASTIC_API_KEY (or ELASTICSEARCH_API_KEY), \
                 or set both ELASTIC_USERNAME and ELASTIC_PASSWORD."
                    .to_string(),
            );
        };

        let client = Client::new();

        Ok(Self {
            client,
            base_url,
            auth,
        })
    }

    fn apply_auth(&self, builder: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            Auth::ApiKey(key) => builder.header("Authorization", format!("ApiKey {}", key)),
            Auth::Basic { username, password } => builder.basic_auth(username, Some(password)),
        }
    }

    pub async fn get(&self, path: &str) -> Result<Response, String> {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn post(&self, path: &str, body: &str) -> Result<Response, String> {
        let url = format!("{}{}", self.base_url, path);
        self.apply_auth(self.client.post(&url))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())
    }
}
