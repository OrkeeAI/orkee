// ABOUTME: Export service for PRD generation in multiple formats
// ABOUTME: Supports Markdown, HTML, PDF (via HTML), and DOCX export formats

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tracing::{error, info, warn};

use crate::error::{IdeateError, Result};
use crate::prd_generator::{GeneratedPRD, PRDGenerator};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Html,
    Pdf,
    Docx,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Markdown => write!(f, "markdown"),
            ExportFormat::Html => write!(f, "html"),
            ExportFormat::Pdf => write!(f, "pdf"),
            ExportFormat::Docx => write!(f, "docx"),
        }
    }
}

/// Export result with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub content: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: usize,
}

/// Export options for customizing output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_toc: bool,
    pub include_metadata: bool,
    pub include_page_numbers: bool, // HTML/PDF only
    pub custom_css: Option<String>, // HTML/PDF only
    pub title: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Markdown,
            include_toc: true,
            include_metadata: true,
            include_page_numbers: false,
            custom_css: None,
            title: None,
        }
    }
}

/// PRD Export Service
pub struct ExportService {
    pool: Pool<Sqlite>,
}

impl ExportService {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Export PRD in specified format
    pub async fn export_prd(
        &self,
        prd: &GeneratedPRD,
        options: ExportOptions,
        session_id: Option<&str>,
    ) -> Result<ExportResult> {
        info!("Exporting PRD in {} format", options.format);

        let content = match options.format {
            ExportFormat::Markdown => self.export_markdown(prd, &options)?,
            ExportFormat::Html => self.export_html(prd, &options).await?,
            ExportFormat::Pdf => {
                // PDF export via HTML conversion
                let html = self.export_html(prd, &options).await?;
                self.export_pdf_from_html(&html)?
            }
            ExportFormat::Docx => self.export_docx(prd, &options)?,
        };

        let file_name = self.generate_filename(prd, &options, session_id);
        let mime_type = self.get_mime_type(&options.format);
        let size_bytes = content.len();

        // Optionally save export record to database
        if let Some(sid) = session_id {
            self.save_export_record(sid, &file_name, &options.format)
                .await
                .ok(); // Don't fail export if record save fails
        }

        Ok(ExportResult {
            format: options.format,
            content,
            file_name,
            mime_type,
            size_bytes,
        })
    }

    /// Export as Markdown
    fn export_markdown(&self, prd: &GeneratedPRD, options: &ExportOptions) -> Result<String> {
        let generator = PRDGenerator::new(self.pool.clone());
        let mut markdown = generator.format_prd_markdown(prd);

        // Add table of contents if requested
        if options.include_toc {
            markdown = self.add_markdown_toc(&markdown);
        }

        // Add metadata header if requested
        if options.include_metadata {
            let title = options
                .title
                .clone()
                .unwrap_or_else(|| "Product Requirements Document".to_string());
            let meta = self.generate_metadata_header(&title);
            markdown = format!("{}\n\n{}", meta, markdown);
        }

        Ok(markdown)
    }

    /// Export as HTML
    async fn export_html(&self, prd: &GeneratedPRD, options: &ExportOptions) -> Result<String> {
        // First generate markdown
        let markdown = self.export_markdown(prd, options)?;

        // Convert markdown to HTML using markdown parser
        let html_body = markdown_to_html(&markdown);

        // Build complete HTML document
        let title = options
            .title
            .clone()
            .unwrap_or_else(|| "Product Requirements Document".to_string());

        let css = if let Some(custom) = &options.custom_css {
            custom.clone()
        } else {
            DEFAULT_PRD_CSS.to_string()
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>{}</style>
</head>
<body>
    <div class="container">
        {}
    </div>
</body>
</html>"#,
            html_escape(&title),
            css,
            html_body
        );

        Ok(html)
    }

    /// Export as PDF (via HTML conversion)
    fn export_pdf_from_html(&self, _html: &str) -> Result<String> {
        // NOTE: PDF generation requires external dependencies (headless Chrome, wkhtmltopdf, etc.)
        // For MVP, we return the HTML with a note that PDF generation requires setup

        warn!("PDF export requested but not fully implemented - returning HTML");

        // In production, this would call a PDF generation library or service:
        // Example using headless Chrome:
        // let pdf_bytes = chrome.generate_pdf(html)?;
        // return Ok(base64::encode(pdf_bytes));

        Err(IdeateError::InvalidInput(
            "PDF export requires additional setup. Please use HTML export and convert to PDF using your browser's Print to PDF feature.".to_string()
        ))
    }

    /// Export as DOCX
    fn export_docx(&self, _prd: &GeneratedPRD, _options: &ExportOptions) -> Result<String> {
        // NOTE: DOCX generation requires the docx crate or similar library
        // For MVP, we provide structure but not full implementation

        warn!("DOCX export requested but not fully implemented");

        // In production, this would use the docx crate:
        // let mut doc = Docx::new();
        // // Add PRD content to document...
        // let bytes = doc.build()?;
        // return Ok(base64::encode(bytes));

        Err(IdeateError::InvalidInput(
            "DOCX export requires additional setup. Please use Markdown or HTML export."
                .to_string(),
        ))
    }

    /// Add table of contents to markdown
    fn add_markdown_toc(&self, markdown: &str) -> String {
        let mut toc = String::from("## Table of Contents\n\n");
        let lines: Vec<&str> = markdown.lines().collect();

        for line in &lines {
            if line.starts_with("## ") {
                let title = line.trim_start_matches("## ").trim();
                let anchor = title
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                toc.push_str(&format!("- [{}](#{})\n", title, anchor));
            }
        }

        toc.push_str("\n---\n\n");
        toc.push_str(markdown);
        toc
    }

    /// Generate metadata header
    fn generate_metadata_header(&self, title: &str) -> String {
        let now = chrono::Utc::now();
        format!(
            r#"---
title: {}
date: {}
generated_by: Orkee Ideate
---"#,
            title,
            now.format("%Y-%m-%d")
        )
    }

    /// Generate filename for export
    fn generate_filename(
        &self,
        prd: &GeneratedPRD,
        options: &ExportOptions,
        session_id: Option<&str>,
    ) -> String {
        let title = if let Some(t) = &options.title {
            slugify(t)
        } else if let Some(overview) = &prd.overview {
            slugify(&overview.problem_statement)
        } else if let Some(sid) = session_id {
            format!("prd-{}", &sid[..8])
        } else {
            "prd".to_string()
        };

        let extension = match options.format {
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
            ExportFormat::Docx => "docx",
        };

        format!("{}.{}", title, extension)
    }

    /// Get MIME type for format
    fn get_mime_type(&self, format: &ExportFormat) -> String {
        match format {
            ExportFormat::Markdown => "text/markdown".to_string(),
            ExportFormat::Html => "text/html".to_string(),
            ExportFormat::Pdf => "application/pdf".to_string(),
            ExportFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string()
            }
        }
    }

    /// Save export record to database
    async fn save_export_record(
        &self,
        session_id: &str,
        file_name: &str,
        format: &ExportFormat,
    ) -> Result<()> {
        let id = nanoid::nanoid!(16);
        let format_str = format.to_string();

        sqlx::query(
            "INSERT INTO ideate_exports (id, session_id, format, file_path, exported_at)
             VALUES (?, ?, ?, ?, datetime('now', 'utc'))",
        )
        .bind(&id)
        .bind(session_id)
        .bind(&format_str)
        .bind(file_name)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save export record: {}", e);
            IdeateError::AIService(format!("Failed to save export record: {}", e))
        })?;

        info!("Saved export record for session {}", session_id);
        Ok(())
    }
}

/// Convert markdown to HTML using a simple parser
fn markdown_to_html(markdown: &str) -> String {
    // NOTE: In production, use a proper markdown parser like pulldown-cmark
    // For MVP, we do basic conversions
    let mut html = String::new();
    let lines: Vec<&str> = markdown.lines().collect();
    let mut in_code_block = false;

    for line in lines {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                html.push_str("<pre><code>");
            } else {
                html.push_str("</code></pre>\n");
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        if let Some(stripped) = line.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", html_escape(stripped)));
        } else if let Some(stripped) = line.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(stripped)));
        } else if let Some(stripped) = line.strip_prefix("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", html_escape(stripped)));
        } else if let Some(stripped) = line.strip_prefix("#### ") {
            html.push_str(&format!("<h4>{}</h4>\n", html_escape(stripped)));
        } else if let Some(stripped) = line.strip_prefix("- ") {
            html.push_str(&format!("<li>{}</li>\n", html_escape(stripped)));
        } else if line.starts_with("**") && line.ends_with("**") {
            let content = &line[2..line.len() - 2];
            html.push_str(&format!("<strong>{}</strong>\n", html_escape(content)));
        } else if !line.trim().is_empty() {
            html.push_str(&format!("<p>{}</p>\n", html_escape(line)));
        } else {
            html.push_str("<br/>\n");
        }
    }

    html
}

/// HTML escape helper
fn html_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Slugify text for filenames
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .take(50) // Limit filename length
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Default CSS for HTML export
const DEFAULT_PRD_CSS: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
    background: #fff;
}

.container {
    background: white;
    padding: 2rem;
}

h1 {
    color: #1a1a1a;
    border-bottom: 3px solid #e1e4e8;
    padding-bottom: 0.5rem;
    font-size: 2.5rem;
}

h2 {
    color: #24292e;
    border-bottom: 1px solid #e1e4e8;
    padding-bottom: 0.3rem;
    margin-top: 2rem;
    font-size: 2rem;
}

h3 {
    color: #24292e;
    margin-top: 1.5rem;
    font-size: 1.5rem;
}

h4 {
    color: #586069;
    margin-top: 1rem;
    font-size: 1.25rem;
}

p {
    margin: 1rem 0;
}

ul, ol {
    margin: 1rem 0;
    padding-left: 2rem;
}

li {
    margin: 0.5rem 0;
}

code {
    background: #f6f8fa;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'SF Mono', Monaco, Consolas, monospace;
    font-size: 0.9em;
}

pre {
    background: #f6f8fa;
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
    margin: 1rem 0;
}

pre code {
    background: none;
    padding: 0;
}

strong {
    font-weight: 600;
    color: #1a1a1a;
}

a {
    color: #0366d6;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

@media print {
    body {
        max-width: none;
        padding: 0;
    }

    .container {
        padding: 0;
    }
}
"#;
