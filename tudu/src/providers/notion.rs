#![allow(dead_code)]

use super::IssueProvider;
use crate::NotionConfig;
use async_trait::async_trait;

// What we send to Notion
#[derive(serde::Serialize)]
struct DatabaseQuery {
    filter: PropertyFilter,
}

#[derive(serde::Serialize)]
struct PropertyFilter {
    property: String,
    // This will depend on the property type - let's start simple
    rich_text: TextFilter,
}

#[derive(serde::Serialize)]
struct TextFilter {
    equals: String,
}

// What we get back from Notion
#[derive(serde::Deserialize)]
struct QueryResponse {
    results: Vec<serde_json::Value>, // Start simple, we just need to check if empty
}

#[derive(Debug)]
pub enum NotionError {
    Http(reqwest::Error),
    Auth,
    NotFound,
}

impl std::fmt::Display for NotionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotionError::Http(e) => write!(f, "HTTP error: {}", e),
            NotionError::Auth => write!(f, "Authentication failed"),
            NotionError::NotFound => write!(f, "Page not found"),
        }
    }
}

impl std::error::Error for NotionError {}

pub struct NotionProvider {
    client: reqwest::Client,
    database_id: String,
}

impl NotionProvider {
    pub fn new(config: &NotionConfig) -> Result<Self, NotionError> {
        // Get token from environment
        let token = std::env::var("NOTION_TOKEN").map_err(|_| NotionError::Auth)?;

        // Build client (same as before)
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            "Authorization",
            auth_value.parse().map_err(|_| NotionError::Auth)?,
        );
        headers.insert(
            "Notion-Version",
            "2022-06-28".parse().map_err(|_| NotionError::Auth)?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(NotionError::Http)?;

        Ok(Self {
            client,
            database_id: config.database_id.clone(), // Store the database ID
        })
    }
}

#[async_trait]
impl IssueProvider for NotionProvider {
    type Error = NotionError;

    async fn issue_exists(&self, id: &str) -> Result<bool, Self::Error> {
        // println!("Printing DB schema {}", id);
        // let schema_url = format!("https://api.notion.com/v1/databases/{}", self.database_id);
        // println!("Getting database schema from: {}", schema_url);
        // let schema_response = self.client
        //     .get(&schema_url)
        //     .send()
        //     .await
        //     .map_err(NotionError::Http)?;
        // println!("Schema response status: {}", schema_response.status());
        // let schema_text = schema_response.text().await.map_err(NotionError::Http)?;
        // println!("Database schema: {}", schema_text);

        // println!("Checking existence of task with ID: {}", id);

        let number = if let Some(dash_pos) = id.find('-') {
            let number_str = &id[dash_pos + 1..];
            match number_str.parse::<i32>() {
                Ok(num) => num,
                Err(_) => {
                    // println!("Could not parse number from ID: {}", id);
                    return Ok(false);
                }
            }
        } else {
            println!("ID format doesn't contain dash: {}", id);
            return Ok(false);
        };

        // We need to know which database to query and which property contains the ID
        // For now, let's hardcode - we'll make this configurable later
        let database_id = &self.database_id;
        let id_property_name = "ID"; // Or whatever your ID property is called

        let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);

        // Build the query
        let query = serde_json::json!({
            "filter": {
                "property": id_property_name,
                "unique_id": {
                    "equals": number
                }
            }
        });
        // let query = serde_json::json!({});

        // Make the request
        let response = self
            .client
            .post(&url)
            .json(&query) // This serializes our struct to JSON
            .send()
            .await
            .map_err(NotionError::Http)?;
        // println!("Response status: {}", response.status());
        // let response_text = response.text().await.map_err(NotionError::Http)?;
        // println!("Response body: {}", response_text);

        match response.status() {
            reqwest::StatusCode::OK => {
                let query_result: QueryResponse =
                    response.json().await.map_err(NotionError::Http)?;

                // println!("Found {} results for {}", query_result.results.len(), id);
                Ok(!query_result.results.is_empty())
            }
            reqwest::StatusCode::BAD_REQUEST => {
                // 400 - Bad request format, let's see what's wrong
                let error_text = response.text().await.map_err(NotionError::Http)?;
                // println!("400 Bad Request for {}: {}", id, error_text);
                Err(NotionError::Auth) // This is actually a query format error, not auth
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                // println!("401 Unauthorized for {}", id);
                Err(NotionError::Auth)
            }
            reqwest::StatusCode::NOT_FOUND => {
                // This probably won't happen for database queries, but just in case
                // println!("404 Not Found for {}", id);
                Ok(false)
            }
            status => {
                let error_text = response.text().await.map_err(NotionError::Http)?;
                // println!("Unexpected status {} for {}: {}", status, id, error_text);
                Err(NotionError::Auth) // Temporary - we'll improve this later
            }
        }
    }
}
