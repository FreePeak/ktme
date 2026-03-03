use super::{
    config::NotionConfig, Document, DocumentMetadata, DocumentProvider, PublishResult,
};
use crate::error::{KtmeError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct NotionProvider {
    config: NotionConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotionPage {
    id: String,
    created_time: String,
    last_edited_time: String,
    url: String,
    parent: NotionParent,
    properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    archived: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum NotionParent {
    PageId { page_id: String },
    DatabaseId { database_id: String },
    Workspace { workspace: bool },
}

impl NotionParent {
    fn page_id(&self) -> Option<&String> {
        match self {
            NotionParent::PageId { page_id } => Some(page_id),
            NotionParent::DatabaseId { database_id } => Some(database_id),
            NotionParent::Workspace { .. } => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NotionBlock {
    id: String,
    #[serde(rename = "type")]
    block_type: String,
    has_children: bool,
    #[serde(default)]
    paragraph: Option<ParagraphBlock>,
    #[serde(default)]
    heading_1: Option<HeadingBlock>,
    #[serde(default)]
    heading_2: Option<HeadingBlock>,
    #[serde(default)]
    heading_3: Option<HeadingBlock>,
    #[serde(default)]
    bulleted_list_item: Option<ListItemBlock>,
    #[serde(default)]
    numbered_list_item: Option<ListItemBlock>,
    #[serde(default)]
    code: Option<CodeBlock>,
    #[serde(default)]
    quote: Option<QuoteBlock>,
    #[serde(default)]
    divider: Option<serde_json::Value>,
    #[serde(default)]
    image: Option<ImageBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParagraphBlock {
    rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeadingBlock {
    rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListItemBlock {
    rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeBlock {
    rich_text: Vec<RichText>,
    language: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuoteBlock {
    rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageBlock {
    file: Option<ImageFile>,
    external: Option<ExternalImage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageFile {
    url: String,
    expiry_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExternalImage {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RichText {
    plain_text: String,
    text: Option<TextLink>,
    #[serde(default)]
    annotations: Annotations,
    #[serde(default)]
    href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TextLink {
    url: Option<String>,
    page: Option<PageRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageRef {
    id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Annotations {
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    strikethrough: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    code: bool,
    #[serde(default)]
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockChildrenResponse {
    results: Vec<NotionBlock>,
    #[serde(rename = "next_cursor")]
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<NotionPage>,
    #[serde(rename = "next_cursor")]
    next_cursor: Option<String>,
    has_more: bool,
}

impl NotionProvider {
    pub fn new(config: NotionConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://api.notion.com/v1/{}", path.trim_start_matches('/'))
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
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Notion-Version", "2022-06-28")
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
                "Notion API error: {} - {}",
                status, error_text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| KtmeError::DeserializationError(e.to_string()))
    }

    async fn get_page(&self, page_id: &str) -> Result<Option<NotionPage>> {
        match self
            .make_request::<NotionPage>(reqwest::Method::GET, &format!("pages/{}", page_id), None)
            .await
        {
            Ok(page) => Ok(Some(page)),
            Err(KtmeError::ApiError(msg)) if msg.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_page_blocks(&self, page_id: &str) -> Result<Vec<NotionBlock>> {
        let mut all_blocks = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let endpoint = if let Some(cursor) = &cursor {
                format!("blocks/{}/children?start_cursor={}", page_id, cursor)
            } else {
                format!("blocks/{}/children", page_id)
            };

            let response: BlockChildrenResponse = self
                .make_request(reqwest::Method::GET, &endpoint, None)
                .await?;

            all_blocks.extend(response.results);

            if !response.has_more {
                break;
            }
            cursor = response.next_cursor;
        }

        Ok(all_blocks)
    }

    fn convert_blocks_to_markdown(&self, blocks: Vec<NotionBlock>) -> String {
        let mut markdown = String::new();

        for block in blocks {
            match block.block_type.as_str() {
                "heading_1" => {
                    if let Some(heading) = block.heading_1 {
                        markdown.push_str(&format!(
                            "# {}\n\n",
                            Self::rich_text_to_string(heading.rich_text)
                        ));
                    }
                }
                "heading_2" => {
                    if let Some(heading) = block.heading_2 {
                        markdown.push_str(&format!(
                            "## {}\n\n",
                            Self::rich_text_to_string(heading.rich_text)
                        ));
                    }
                }
                "heading_3" => {
                    if let Some(heading) = block.heading_3 {
                        markdown.push_str(&format!(
                            "### {}\n\n",
                            Self::rich_text_to_string(heading.rich_text)
                        ));
                    }
                }
                "paragraph" => {
                    if let Some(para) = block.paragraph {
                        let text = Self::rich_text_to_string(para.rich_text);
                        if !text.is_empty() {
                            markdown.push_str(&format!("{}\n\n", text));
                        }
                    }
                }
                "bulleted_list_item" => {
                    if let Some(item) = block.bulleted_list_item {
                        markdown.push_str(&format!(
                            "- {}\n",
                            Self::rich_text_to_string(item.rich_text)
                        ));
                    }
                }
                "numbered_list_item" => {
                    if let Some(item) = block.numbered_list_item {
                        markdown.push_str(&format!(
                            "1. {}\n",
                            Self::rich_text_to_string(item.rich_text)
                        ));
                    }
                }
                "code" => {
                    if let Some(code) = block.code {
                        let lang = code.language;
                        let text = Self::rich_text_to_string(code.rich_text);
                        markdown.push_str(&format!("```{}\n{}\n```\n\n", lang, text));
                    }
                }
                "quote" => {
                    if let Some(quote) = block.quote {
                        let text = Self::rich_text_to_string(quote.rich_text);
                        markdown.push_str(&format!("> {}\n\n", text));
                    }
                }
                "divider" => {
                    markdown.push_str("---\n\n");
                }
                "image" => {
                    if let Some(image) = block.image {
                        let url = image.file.map(|f| f.url).or(image.external.map(|e| e.url));
                        if let Some(url) = url {
                            markdown.push_str(&format!("![Image]({})\n\n", url));
                        }
                    }
                }
                _ => {}
            }
        }

        markdown
    }

    fn rich_text_to_string(rich_text: Vec<RichText>) -> String {
        rich_text
            .into_iter()
            .map(|rt| {
                let text = rt.plain_text;
                let mut result = text;
                if rt.annotations.bold {
                    result = format!("**{}**", result);
                }
                if rt.annotations.italic {
                    result = format!("*{}*", result);
                }
                if rt.annotations.strikethrough {
                    result = format!("~~{}~~", result);
                }
                if rt.annotations.underline {
                    result = format!("<u>{}</u>", result);
                }
                if rt.annotations.code {
                    result = format!("`{}`", result);
                }
                result
            })
            .collect()
    }

    fn get_page_title(&self, page: &NotionPage) -> String {
        if let Some(props) = &page.properties {
            for prop in props.values() {
                if let Some(title) = prop.get("title") {
                    if let Some(arr) = title.as_array() {
                        return arr
                            .iter()
                            .filter_map(|v| v.get("plain_text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join("");
                    }
                }
            }
        }
        "Untitled".to_string()
    }

    async fn search_by_title(&self, title: &str) -> Result<Option<NotionPage>> {
        let body = serde_json::json!({
            "query": title,
            "filter": {
                "property": "object",
                "value": "page"
            },
            "page_size": 10
        });

        let response: SearchResponse = self
            .make_request(reqwest::Method::POST, "search", Some(body))
            .await?;

        Ok(response
            .results
            .into_iter()
            .find(|p| self.get_page_title(p) == title))
    }
}

#[async_trait]
impl DocumentProvider for NotionProvider {
    fn name(&self) -> &str {
        "notion"
    }

    async fn health_check(&self) -> Result<bool> {
        match self
            .make_request::<serde_json::Value>(reqwest::Method::GET, "users/me", None)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        match self.get_page(id).await {
            Ok(Some(page)) => {
                let blocks = self.get_page_blocks(id).await?;
                let content = self.convert_blocks_to_markdown(blocks);
                let title = self.get_page_title(&page);

                Ok(Some(Document {
                    id: page.id,
                    title,
                    content,
                    url: Some(page.url),
                    parent_id: page.parent.page_id().cloned(),
                    metadata: DocumentMetadata {
                        created_at: Some(page.created_time),
                        updated_at: Some(page.last_edited_time),
                        author: None,
                        version: None,
                        labels: vec![],
                    },
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn find_document(&self, title: &str) -> Result<Option<Document>> {
        match self.search_by_title(title).await {
            Ok(Some(page)) => self.get_document(&page.id).await,
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn create_document(&self, _doc: &Document) -> Result<PublishResult> {
        Err(KtmeError::UnsupportedOperation(
            "Notion create not yet implemented - use Notion UI to create pages".to_string(),
        ))
    }

    async fn update_document(&self, _id: &str, _content: &str) -> Result<PublishResult> {
        Err(KtmeError::UnsupportedOperation(
            "Notion update not yet implemented".to_string(),
        ))
    }

    async fn update_section(
        &self,
        _id: &str,
        _section: &str,
        _content: &str,
    ) -> Result<PublishResult> {
        Err(KtmeError::UnsupportedOperation(
            "Notion section update not yet implemented".to_string(),
        ))
    }

    async fn delete_document(&self, _id: &str) -> Result<()> {
        Err(KtmeError::UnsupportedOperation(
            "Notion delete not yet implemented".to_string(),
        ))
    }

    async fn list_documents(&self, _container: &str) -> Result<Vec<Document>> {
        let body = serde_json::json!({
            "filter": {
                "property": "object",
                "value": "page"
            },
            "page_size": 100
        });

        let response: SearchResponse = self
            .make_request(reqwest::Method::POST, "search", Some(body))
            .await?;

        let mut documents = Vec::new();
        for page in response.results {
            if let Ok(Some(doc)) = self.get_document(&page.id).await {
                documents.push(doc);
            }
        }

        Ok(documents)
    }

    async fn search_documents(&self, query: &str) -> Result<Vec<Document>> {
        let body = serde_json::json!({
            "query": query,
            "filter": {
                "property": "object",
                "value": "page"
            },
            "page_size": 50
        });

        let response: SearchResponse = self
            .make_request(reqwest::Method::POST, "search", Some(body))
            .await?;

        let mut documents = Vec::new();
        for page in response.results {
            if let Ok(Some(doc)) = self.get_document(&page.id).await {
                documents.push(doc);
            }
        }

        Ok(documents)
    }

    fn config(&self) -> &super::config::ProviderConfig {
        static DEFAULT_CONFIG: std::sync::OnceLock<super::config::ProviderConfig> =
            std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| super::config::ProviderConfig {
            id: 0,
            provider_type: "notion".to_string(),
            config: serde_json::to_value(&self.config).unwrap(),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

pub fn compute_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notion_provider_creation() {
        let config = NotionConfig {
            api_key: "secret_xxxx".to_string(),
            workspace_id: Some("workspace_123".to_string()),
            workspace_name: Some("Test Workspace".to_string()),
            parent_page_id: None,
            sync_enabled: true,
        };

        let provider = NotionProvider::new(config);
        assert_eq!(provider.name(), "notion");
    }

    #[test]
    fn test_hash_computation() {
        let content = "Hello, World!";
        let hash = compute_hash(content);
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_rich_text_to_string() {
        let rich_text = vec![
            RichText {
                plain_text: "Hello".to_string(),
                text: None,
                annotations: Annotations {
                    bold: true,
                    italic: false,
                    strikethrough: false,
                    underline: false,
                    code: false,
                    color: "default".to_string(),
                },
                href: None,
            },
            RichText {
                plain_text: " World".to_string(),
                text: None,
                annotations: Annotations::default(),
                href: None,
            },
        ];

        let result = NotionProvider::rich_text_to_string(rich_text);
        assert_eq!(result, "**Hello** World");
    }
}
