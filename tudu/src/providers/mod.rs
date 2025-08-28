use std::error::Error;

#[async_trait::async_trait]
pub trait IssueProvider {
    type Error: Error + Send + Sync + 'static;

    // MVP: just check existence
    async fn issue_exists(&self, id: &str) -> Result<bool, Self::Error>;

    // Future: full issue data
    // async fn get_issue(&self, id: &str) -> Result<Issue, Self::Error>;
}

// Re-export our providers
pub mod notion;
pub use notion::NotionProvider;
