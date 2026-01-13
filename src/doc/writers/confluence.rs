use crate::error::{KtmeError, Result};
use serde::{Deserialize, Serialize};

pub struct ConfluenceWriter {
    base_url: String,
    api_token: String,
    space_key: String,
    client: reqwest::Client,
}

impl ConfluenceWriter {
    pub fn new(base_url: String, api_token: String, space_key: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme-cli")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            api_token,
            space_key,
            client,
        }
    }

    pub async fn create_page(&self, title: &str, content: &str) -> Result<String> {
        tracing::info!("Creating Confluence page: {}", title);

        // Convert Markdown to Confluence Storage Format (XHTML)
        let storage_content = Self::markdown_to_storage_format(content);

        // Prepare request body
        let body = CreatePageRequest {
            r#type: "page".to_string(),
            title: title.to_string(),
            space: SpaceKey {
                key: self.space_key.clone(),
            },
            body: PageBody {
                storage: StorageContent {
                    value: storage_content,
                    representation: "storage".to_string(),
                },
            },
        };

        // Send request
        let url = format!("{}/rest/api/content", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                KtmeError::NetworkError(format!("Failed to create Confluence page: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(KtmeError::Confluence(format!(
                "Failed to create page ({}): {}",
                status, error_body
            )));
        }

        let created_page: CreatePageResponse = response.json().await.map_err(|e| {
            KtmeError::DeserializationError(format!("Failed to parse response: {}", e))
        })?;

        tracing::info!("Created Confluence page with ID: {}", created_page.id);
        Ok(created_page.id)
    }

    pub async fn update_page(&self, page_id: &str, content: &str) -> Result<()> {
        tracing::info!("Updating Confluence page: {}", page_id);

        // First, get the current page to retrieve version number
        let get_url = format!("{}/rest/api/content/{}", self.base_url, page_id);
        let current_page: GetPageResponse = self
            .client
            .get(&get_url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| KtmeError::NetworkError(format!("Failed to get page: {}", e)))?
            .json()
            .await
            .map_err(|e| KtmeError::DeserializationError(format!("Failed to parse page: {}", e)))?;

        // Convert Markdown to Confluence Storage Format
        let storage_content = Self::markdown_to_storage_format(content);

        // Prepare update request
        let body = UpdatePageRequest {
            version: Version {
                number: current_page.version.number + 1,
            },
            title: current_page.title,
            r#type: "page".to_string(),
            body: PageBody {
                storage: StorageContent {
                    value: storage_content,
                    representation: "storage".to_string(),
                },
            },
        };

        // Send update request
        let update_url = format!("{}/rest/api/content/{}", self.base_url, page_id);
        let response = self
            .client
            .put(&update_url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KtmeError::NetworkError(format!("Failed to update page: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(KtmeError::Confluence(format!(
                "Failed to update page ({}): {}",
                status, error_body
            )));
        }

        tracing::info!("Successfully updated Confluence page: {}", page_id);
        Ok(())
    }

    /// Convert Markdown to Confluence Storage Format (basic conversion)
    /// This handles: headings, paragraphs, bold, italic, code blocks, lists
    fn markdown_to_storage_format(markdown: &str) -> String {
        use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

        let mut html = String::new();
        let mut in_list = false;
        let mut list_stack: Vec<bool> = Vec::new(); // true = ordered, false = unordered

        let parser = Parser::new(markdown);

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        let level_num = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        html.push_str(&format!("<h{}>", level_num));
                    }
                    Tag::Paragraph => {
                        if !in_list {
                            html.push_str("<p>");
                        }
                    }
                    Tag::Strong => html.push_str("<strong>"),
                    Tag::Emphasis => html.push_str("<em>"),
                    Tag::CodeBlock(kind) => match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                            html.push_str(&format!("<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">{}</ac:parameter><ac:plain-text-body><![CDATA[", lang));
                        }
                        _ => {
                            html.push_str("<ac:structured-macro ac:name=\"code\"><ac:plain-text-body><![CDATA[");
                        }
                    },
                    Tag::List(start) => {
                        let is_ordered = start.is_some();
                        list_stack.push(is_ordered);
                        in_list = true;
                        if is_ordered {
                            html.push_str("<ol>");
                        } else {
                            html.push_str("<ul>");
                        }
                    }
                    Tag::Item => html.push_str("<li>"),
                    Tag::Link { dest_url, .. } => {
                        html.push_str(&format!(
                            "<a href=\"{}\">",
                            html_escape::encode_text(&dest_url)
                        ));
                    }
                    Tag::Image { dest_url, .. } => {
                        html.push_str(&format!(
                            "<ac:image><ri:url ri:value=\"{}\" /></ac:image>",
                            html_escape::encode_text(&dest_url)
                        ));
                    }
                    Tag::BlockQuote => html.push_str("<blockquote>"),
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(level) => {
                        let level_num = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        html.push_str(&format!("</h{}>", level_num));
                    }
                    TagEnd::Paragraph => {
                        if !in_list {
                            html.push_str("</p>");
                        }
                    }
                    TagEnd::Strong => html.push_str("</strong>"),
                    TagEnd::Emphasis => html.push_str("</em>"),
                    TagEnd::CodeBlock => {
                        html.push_str("]]></ac:plain-text-body></ac:structured-macro>");
                    }
                    TagEnd::List(_) => {
                        if let Some(is_ordered) = list_stack.pop() {
                            if is_ordered {
                                html.push_str("</ol>");
                            } else {
                                html.push_str("</ul>");
                            }
                        }
                        if list_stack.is_empty() {
                            in_list = false;
                        }
                    }
                    TagEnd::Item => html.push_str("</li>"),
                    TagEnd::Link => html.push_str("</a>"),
                    TagEnd::BlockQuote => html.push_str("</blockquote>"),
                    _ => {}
                },
                Event::Text(text) => {
                    html.push_str(&html_escape::encode_text(&text));
                }
                Event::Code(code) => {
                    html.push_str(&format!("<code>{}</code>", html_escape::encode_text(&code)));
                }
                Event::SoftBreak => html.push(' '),
                Event::HardBreak => html.push_str("<br/>"),
                Event::Rule => html.push_str("<hr/>"),
                _ => {}
            }
        }

        html
    }
}

// Request/Response structures
#[derive(Debug, Serialize)]
struct CreatePageRequest {
    r#type: String,
    title: String,
    space: SpaceKey,
    body: PageBody,
}

#[derive(Debug, Serialize)]
struct SpaceKey {
    key: String,
}

#[derive(Debug, Serialize)]
struct PageBody {
    storage: StorageContent,
}

#[derive(Debug, Serialize)]
struct StorageContent {
    value: String,
    representation: String,
}

#[derive(Debug, Deserialize)]
struct CreatePageResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GetPageResponse {
    title: String,
    version: Version,
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    number: u32,
}

#[derive(Debug, Serialize)]
struct UpdatePageRequest {
    version: Version,
    title: String,
    r#type: String,
    body: PageBody,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_storage_basic() {
        let markdown = "# Heading\n\nThis is a paragraph.";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<h1>"));
        assert!(result.contains("</h1>"));
        assert!(result.contains("<p>"));
        assert!(result.contains("This is a paragraph."));
    }

    #[test]
    fn test_markdown_to_storage_bold_italic() {
        let markdown = "**bold** and *italic*";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_markdown_to_storage_code_block() {
        let markdown = "```rust\nfn main() {}\n```";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<ac:structured-macro ac:name=\"code\">"));
        assert!(result.contains("rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_markdown_to_storage_list() {
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>"));
        assert!(result.contains("Item 1"));
    }

    #[test]
    fn test_markdown_to_storage_ordered_list() {
        let markdown = "1. First\n2. Second\n3. Third";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>"));
        assert!(result.contains("First"));
    }

    #[test]
    fn test_markdown_to_storage_inline_code() {
        let markdown = "Use `code` inline.";
        let result = ConfluenceWriter::markdown_to_storage_format(markdown);

        assert!(result.contains("<code>code</code>"));
    }
}
