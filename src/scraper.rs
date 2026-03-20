use scraper::{Html, Node, Selector};

use crate::error::Error;
use crate::types::{FileFormat, Md5, Metadata, SearchResponse, SearchResult};

// ── CSS selectors ────────────────────────────────────────────────
// Centralised here so they are easy to update when Anna's Archive
// changes their HTML structure.

const SEL_RESULT: &str = "div.flex.pt-3.pb-3.border-b";
const SEL_MD5_LINK: &str = "a[href^=\"/md5/\"]";
const SEL_TITLE: &str = "a.js-vim-focus";
const SEL_METADATA: &str = "div.text-gray-800.font-semibold.text-sm";
const SEL_AUTHOR_ICON: &str = "span.icon-\\[mdi--user-edit\\]";
const SEL_PAGINATION: &str = "div.uppercase.text-xs.text-gray-500";

impl SearchResponse {
    /// Construct a `SearchResponse` from an HTML search results page.
    pub fn from_html(html: &str, page: u32) -> Result<Self, Error> {
        let document = Html::parse_document(html);

        let result_sel = selector(SEL_RESULT)?;
        let link_sel = selector(SEL_MD5_LINK)?;
        let title_sel = selector(SEL_TITLE)?;
        let meta_sel = selector(SEL_METADATA)?;
        let author_icon_sel = selector(SEL_AUTHOR_ICON)?;
        let a_sel = selector("a")?;

        let mut results = Vec::new();

        for elem in document.select(&result_sel) {
            // Extract MD5 from first /md5/ link.
            let md5 = elem
                .select(&link_sel)
                .next()
                .and_then(|a| a.value().attr("href"))
                .and_then(|href| href.strip_prefix("/md5/"))
                .map(Md5::from);

            let Some(md5) = md5 else { continue };

            // Title from a.js-vim-focus.
            let title = elem
                .select(&title_sel)
                .next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // Author: link containing the user-edit icon.
            let author = elem
                .select(&a_sel)
                .find(|a| a.select(&author_icon_sel).next().is_some())
                .map(|a| a.text().collect::<String>().trim().to_string())
                .filter(|s| !s.is_empty());

            // Metadata line (format · size · language · year).
            let meta = elem
                .select(&meta_sel)
                .next()
                .map(|div| Metadata::from_line(&extract_text_without_scripts(div)))
                .unwrap_or_default();

            results.push(SearchResult {
                md5,
                title,
                author,
                format: meta.format,
                size: meta.size,
                language: meta.language,
            });
        }

        let has_more = detect_has_more(&document);
        Ok(Self {
            results,
            page,
            has_more,
        })
    }
}

impl Metadata {
    /// Parse the "format · size · language · year" metadata line.
    pub fn from_line(text: &str) -> Self {
        let parts: Vec<&str> = text.split('·').map(|s| s.trim()).collect();

        let mut meta = Self::default();

        for part in parts {
            let candidate = FileFormat::from(part);
            if candidate.is_known() {
                meta.format = Some(candidate);
            } else if is_file_size(part) {
                meta.size = Some(part.to_string());
            } else if part.contains('[') && part.contains(']') {
                meta.language = Some(part.to_string());
            }
        }

        meta
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn selector(sel: &'static str) -> Result<Selector, Error> {
    Selector::parse(sel).map_err(|_| Error::Selector { selector: sel })
}

/// Collect text from an element, skipping `<script>` descendants.
fn extract_text_without_scripts(element: scraper::ElementRef) -> String {
    let mut text = String::new();
    for node in element.descendants() {
        if let Node::Text(t) = node.value() {
            let in_script = node.ancestors().any(|ancestor| {
                ancestor
                    .value()
                    .as_element()
                    .is_some_and(|el| el.name() == "script")
            });
            if !in_script {
                text.push_str(t);
            }
        }
    }
    text
}

/// Returns true if the string looks like a file size (e.g. "6.4MB").
fn is_file_size(s: &str) -> bool {
    let s = s.trim().to_lowercase();
    let units = ["gb", "mb", "kb", "b"];
    let Some(unit) = units.iter().find(|u| s.ends_with(*u)) else {
        return false;
    };
    let number_part = &s[..s.len() - unit.len()];
    number_part.chars().any(|c| c.is_ascii_digit())
}

/// Check pagination text for "more results" indicators.
fn detect_has_more(document: &Html) -> bool {
    let Some(sel) = Selector::parse(SEL_PAGINATION).ok() else {
        return false;
    };

    for elem in document.select(&sel) {
        let text = elem.text().collect::<String>().to_lowercase();
        if text.contains("total")
            && (text.contains('+') || text.contains("more"))
        {
            return true;
        }
    }

    false
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_from_line_full() {
        let meta =
            Metadata::from_line("PDF · 54.2MB · English [en] · 1987");
        assert_eq!(meta.format, Some(FileFormat::Pdf));
        assert_eq!(meta.size.as_deref(), Some("54.2MB"));
        assert_eq!(meta.language.as_deref(), Some("English [en]"));
    }

    #[test]
    fn test_metadata_from_line_partial() {
        let meta = Metadata::from_line("epub · 1.2MB");
        assert_eq!(meta.format, Some(FileFormat::Epub));
        assert_eq!(meta.size.as_deref(), Some("1.2MB"));
        assert!(meta.language.is_none());
    }

    #[test]
    fn test_metadata_from_line_empty() {
        let meta = Metadata::from_line("");
        assert!(meta.format.is_none());
        assert!(meta.size.is_none());
        assert!(meta.language.is_none());
    }

    #[test]
    fn test_is_file_size() {
        assert!(is_file_size("54.2MB"));
        assert!(is_file_size("1.2GB"));
        assert!(is_file_size("512KB"));
        assert!(is_file_size("1024B"));
        assert!(!is_file_size("zlib"));
        assert!(!is_file_size("English"));
        assert!(!is_file_size(""));
    }

    #[test]
    fn test_file_format_from_str() {
        assert_eq!(FileFormat::from("PDF"), FileFormat::Pdf);
        assert_eq!(FileFormat::from("pdf"), FileFormat::Pdf);
        assert_eq!(FileFormat::from("Epub"), FileFormat::Epub);
        assert_eq!(
            FileFormat::from("xyz"),
            FileFormat::Unknown("xyz".into())
        );
    }

    #[test]
    fn test_search_response_from_empty_html() {
        let resp = SearchResponse::from_html("<html></html>", 1)
            .expect("empty HTML should parse without error");
        assert!(resp.results.is_empty());
        assert!(!resp.has_more);
        assert_eq!(resp.page, 1);
    }
}
