use crate::error::Result;

pub struct GitLabProvider {
    api_token: Option<String>,
}

impl GitLabProvider {
    pub fn new(api_token: Option<String>) -> Self {
        Self { api_token }
    }

    pub async fn fetch_merge_request(&self, project: &str, mr_number: u32) -> Result<()> {
        tracing::info!("Fetching GitLab MR #{} from {}", mr_number, project);
        // TODO: Implement GitLab MR fetching
        Ok(())
    }
}
