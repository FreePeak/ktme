use super::{
    config::ConfluenceConfig, Document, DocumentMetadata, DocumentProvider, PublishResult,
    PublishStatus,
};
use crate::error::{KtmeError, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confluence provider for publishing documentation
pub struct ConfluenceProvider {
    config: ConfluenceConfig,
    client: reqwest::Client,
    auth_header: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfluencePage {
    id: String,
    title: String,
    #[serde(rename = "_expandable")]
    expandable: Option<HashMap<String, String>>,
    #[serde(rename = "_links")]
    links: Option<HashMap<String, String>>,
    version: Option<ConfluenceVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfluenceVersion {
    number: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageContent {
    id: String,
    title: String,
    #[serde(rename = "space")]
    space: ConfluenceSpace,
    body: PageBody,
    #[serde(rename = "type")]
    page_type: String,
    status: String,
    version: Option<ConfluenceVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfluenceSpace {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageBody {
    storage: Storage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Storage {
    value: String,
    representation: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageUpdate {
    id: String,
    #[serde(rename = "type")]
    page_type: String,
    title: String,
    body: PageBody,
    version: ConfluenceVersion,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    size: i32,
    #[serde(rename = "start")]
    start_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    id: String,
    title: String,
    excerpt: Option<String>,
    url: String,
}

impl ConfluenceProvider {
    pub fn new(config: ConfluenceConfig) -> Self {
        let auth = if let Some(token) = &config.api_token {
            let auth_string = format!("{}:{}", config.username, token);
            let encoded = general_purpose::STANDARD.encode(auth_string);
            format!("Basic {}", encoded)
        } else if config.is_cloud {
            // For OAuth or other cloud auth methods
            format!(
                "Bearer {}",
                config.api_token.as_ref().unwrap_or(&String::new())
            )
        } else {
            // For PAT or other auth
            format!(
                "Bearer {}",
                config.api_token.as_ref().unwrap_or(&String::new())
            )
        };

        let client = reqwest::Client::builder()
            .user_agent("ktme/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            auth_header: auth,
        }
    }

    fn api_url(&self, path: &str) -> String {
        let base = self.config.base_url.trim_end_matches('/');
        format!("{}/rest/api/{}", base, path.trim_start_matches('/'))
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        let url = self.api_url(endpoint);

        let mut request = self
            .client
            .request(method, &url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| KtmeError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(KtmeError::ApiError(format!(
                "Confluence API error: {} - {}",
                status, error_text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| KtmeError::DeserializationError(e.to_string()))
    }

    async fn get_page_by_id(&self, page_id: &str) -> Result<Option<PageContent>> {
        let endpoint = format!("content/{}?expand=body.storage,version,space", page_id);

        match self
            .make_request::<PageContent>(reqwest::Method::GET, &endpoint, None)
            .await
        {
            Ok(page) => Ok(Some(page)),
            Err(KtmeError::ApiError(msg)) if msg.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn search_page_by_title(&self, title: &str) -> Result<Option<ConfluencePage>> {
        let query = format!("title=\"{}\" and space={}", title, self.config.space_key);
        let endpoint = format!(
            "content/search?cql={}&limit=1&expand=version,space",
            urlencoding::encode(&query)
        );

        #[derive(Debug, Serialize, Deserialize)]
        struct SearchResponse {
            results: Vec<ConfluencePage>,
            size: i32,
        }

        let response: SearchResponse = self
            .make_request(reqwest::Method::GET, &endpoint, None)
            .await?;

        Ok(response.results.into_iter().next())
    }

    async fn create_page(&self, doc: &Document) -> Result<PageContent> {
        let page = PageContent {
            id: String::new(),
            title: doc.title.clone(),
            space: ConfluenceSpace {
                key: self.config.space_key.clone(),
            },
            body: PageBody {
                storage: Storage {
                    value: doc.content.clone(),
                    representation: "storage".to_string(),
                },
            },
            page_type: "page".to_string(),
            status: "current".to_string(),
            version: None,
        };

        let endpoint = "content";

        // Add parent if specified
        let mut body = serde_json::to_value(&page)
            .map_err(|e| KtmeError::SerializationError(e.to_string()))?;

        if let Some(parent_id) = &doc
            .parent_id
            .as_ref()
            .or(self.config.default_parent_id.as_ref())
        {
            body["ancestors"] = serde_json::json!([{
                "id": parent_id
            }]);
        }

        // Add labels if configured
        if !self.config.default_labels.is_empty() {
            let mut labels = vec![];
            for label in &self.config.default_labels {
                labels.push(serde_json::json!({
                    "prefix": "global",
                    "name": label
                }));
            }
            body["metadata"]["labels"] = serde_json::json!(labels);
        }

        self.make_request(reqwest::Method::POST, endpoint, Some(body))
            .await
    }

    async fn update_page(&self, page_id: &str, doc: &Document) -> Result<PageContent> {
        // Get current page to retrieve version number
        let current_page = self
            .get_page_by_id(page_id)
            .await?
            .ok_or_else(|| KtmeError::DocumentNotFound(page_id.to_string()))?;

        // Increment version
        let new_version = ConfluenceVersion {
            number: current_page.version.as_ref().map_or(1, |v| v.number + 1),
        };

        let update = PageUpdate {
            id: page_id.to_string(),
            page_type: "page".to_string(),
            title: doc.title.clone(),
            body: PageBody {
                storage: Storage {
                    value: doc.content.clone(),
                    representation: "storage".to_string(),
                },
            },
            version: new_version,
        };

        let endpoint = format!("content/{}", page_id);
        let body = serde_json::to_value(&update)
            .map_err(|e| KtmeError::SerializationError(e.to_string()))?;

        self.make_request(reqwest::Method::PUT, &endpoint, Some(body))
            .await
    }

    fn convert_to_document(&self, page: PageContent) -> Document {
        let url = if self.config.is_cloud {
            format!(
                "{}/wiki/spaces/{}/pages/{}",
                self.config.base_url.trim_end_matches('/'),
                page.space.key,
                page.id
            )
        } else {
            format!(
                "{}/pages/viewpage.action?pageId={}",
                self.config.base_url.trim_end_matches('/'),
                page.id
            )
        };

        Document {
            id: page.id,
            title: page.title,
            content: page.body.storage.value,
            url: Some(url),
            parent_id: None,
            metadata: DocumentMetadata {
                created_at: None,
                updated_at: None,
                author: None,
                version: page.version.map(|v| v.number as u32),
                labels: vec![],
            },
        }
    }
}

#[async_trait]
impl DocumentProvider for ConfluenceProvider {
    fn name(&self) -> &str {
        "confluence"
    }

    async fn health_check(&self) -> Result<bool> {
        let endpoint = "space?limit=1";

        match self
            .make_request::<serde_json::Value>(reqwest::Method::GET, endpoint, None)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        match self.get_page_by_id(id).await {
            Ok(Some(page)) => Ok(Some(self.convert_to_document(page))),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn find_document(&self, title: &str) -> Result<Option<Document>> {
        match self.search_page_by_title(title).await {
            Ok(Some(page)) => {
                let page_content = self.get_page_by_id(&page.id).await?;
                Ok(page_content.map(|p| self.convert_to_document(p)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn create_document(&self, doc: &Document) -> Result<PublishResult> {
        // Check if page already exists
        if let Ok(Some(_)) = self.find_document(&doc.title).await {
            return Err(KtmeError::DocumentExists(doc.title.clone()));
        }

        let page = self.create_page(doc).await?;

        let url = if self.config.is_cloud {
            format!(
                "{}/wiki/spaces/{}/pages/{}",
                self.config.base_url.trim_end_matches('/'),
                page.space.key,
                page.id
            )
        } else {
            format!(
                "{}/pages/viewpage.action?pageId={}",
                self.config.base_url.trim_end_matches('/'),
                page.id
            )
        };

        Ok(PublishResult {
            document_id: page.id,
            url,
            version: 1,
            status: PublishStatus::Created,
        })
    }

    async fn update_document(&self, id: &str, content: &str) -> Result<PublishResult> {
        let current_page = self
            .get_page_by_id(id)
            .await?
            .ok_or_else(|| KtmeError::DocumentNotFound(id.to_string()))?;

        // Check if content is the same
        if current_page.body.storage.value == content {
            return Ok(PublishResult {
                document_id: id.to_string(),
                url: String::new(),
                version: current_page
                    .version
                    .as_ref()
                    .map(|v| v.number as u32)
                    .unwrap_or(1),
                status: PublishStatus::NoChanges,
            });
        }

        let doc = Document {
            id: id.to_string(),
            title: current_page.title.clone(),
            content: content.to_string(),
            url: None,
            parent_id: None,
            metadata: DocumentMetadata::default(),
        };

        let updated_page = self.update_page(id, &doc).await?;

        let url = if self.config.is_cloud {
            format!(
                "{}/wiki/spaces/{}/pages/{}",
                self.config.base_url.trim_end_matches('/'),
                updated_page.space.key,
                updated_page.id
            )
        } else {
            format!(
                "{}/pages/viewpage.action?pageId={}",
                self.config.base_url.trim_end_matches('/'),
                updated_page.id
            )
        };

        Ok(PublishResult {
            document_id: id.to_string(),
            url,
            version: updated_page.version.map(|v| v.number as u32).unwrap_or(2),
            status: PublishStatus::Updated,
        })
    }

    async fn update_section(
        &self,
        id: &str,
        section: &str,
        content: &str,
    ) -> Result<PublishResult> {
        let current_page = self
            .get_page_by_id(id)
            .await?
            .ok_or_else(|| KtmeError::DocumentNotFound(id.to_string()))?;

        // For Confluence, we'll append the section content
        let section_header = format!("h2. {}", section);
        let new_content = if current_page.body.storage.value.contains(&section_header) {
            // Replace existing section
            let start_pattern = format!("h2. {}", section);
            let start = current_page
                .body
                .storage
                .value
                .find(&start_pattern)
                .unwrap_or(0);

            let next_h2 = current_page.body.storage.value[start + 1..]
                .find("h2. ")
                .map(|pos| start + 1 + pos);

            if let Some(_end) = next_h2 {
                format!(
                    "{}\n{}\n\n{}",
                    &current_page.body.storage.value[..start],
                    &section_header,
                    content
                )
            } else {
                format!(
                    "{}\n{}\n\n{}",
                    &current_page.body.storage.value[..start],
                    &section_header,
                    content
                )
            }
        } else {
            // Append new section
            format!(
                "{}\n\nh2. {}\n\n{}",
                current_page.body.storage.value, section, content
            )
        };

        self.update_document(id, &new_content).await
    }

    async fn delete_document(&self, id: &str) -> Result<()> {
        let endpoint = format!("content/{}", id);

        self.make_request::<serde_json::Value>(reqwest::Method::DELETE, &endpoint, None)
            .await?;

        Ok(())
    }

    async fn list_documents(&self, container: &str) -> Result<Vec<Document>> {
        let cql = format!("space={}", urlencoding::encode(container));
        let endpoint = format!("content/search?cql={}&expand=version,space&limit=100", cql);

        let response: SearchResponse = self
            .make_request(reqwest::Method::GET, &endpoint, None)
            .await?;

        let mut documents = Vec::new();
        for result in response.results {
            if let Some(page) = self.get_page_by_id(&result.id).await? {
                documents.push(self.convert_to_document(page));
            }
        }

        Ok(documents)
    }

    async fn search_documents(&self, query: &str) -> Result<Vec<Document>> {
        let cql = format!(
            "space={} and text~\"{}\"",
            urlencoding::encode(&self.config.space_key),
            urlencoding::encode(query)
        );
        let endpoint = format!("content/search?cql={}&expand=version,space&limit=50", cql);

        let response: SearchResponse = self
            .make_request(reqwest::Method::GET, &endpoint, None)
            .await?;

        let mut documents = Vec::new();
        for result in response.results {
            if let Some(page) = self.get_page_by_id(&result.id).await? {
                documents.push(self.convert_to_document(page));
            }
        }

        Ok(documents)
    }

    fn config(&self) -> &super::config::ProviderConfig {
        // Return a default config reference
        // In practice, this should be stored during provider creation
        static DEFAULT_CONFIG: std::sync::OnceLock<super::config::ProviderConfig> =
            std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| super::config::ProviderConfig {
            id: 0,
            provider_type: "confluence".to_string(),
            config: serde_json::to_value(&self.config).unwrap(),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confluence_provider_creation() {
        let config = ConfluenceConfig {
            base_url: "https://example.atlassian.net".to_string(),
            username: "test@example.com".to_string(),
            api_token: Some("token".to_string()),
            space_key: "DEV".to_string(),
            default_parent_id: None,
            default_labels: vec!["documentation".to_string()],
            is_cloud: true,
        };

        let provider = ConfluenceProvider::new(config);
        assert_eq!(provider.name(), "confluence");
        assert!(!provider.auth_header.is_empty());
    }

    #[test]
    fn test_api_url_construction() {
        let config = ConfluenceConfig {
            base_url: "https://example.atlassian.net/".to_string(),
            username: "test@example.com".to_string(),
            api_token: Some("token".to_string()),
            space_key: "DEV".to_string(),
            default_parent_id: None,
            default_labels: vec![],
            is_cloud: true,
        };

        let provider = ConfluenceProvider::new(config);
        assert_eq!(
            provider.api_url("content/123"),
            "https://example.atlassian.net/rest/api/content/123"
        );
        assert_eq!(
            provider.api_url("/content/123"),
            "https://example.atlassian.net/rest/api/content/123"
        );
    }
}
