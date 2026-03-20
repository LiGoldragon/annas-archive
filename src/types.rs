use serde::{Deserialize, Serialize};

// ── File format ──────────────────────────────────────────────────

/// Known document file formats.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileFormat {
    Pdf,
    Epub,
    Mobi,
    Azw3,
    Djvu,
    Cbr,
    Cbz,
    Fb2,
    Txt,
    Doc,
    Docx,
    Rtf,
    /// Format not in the known set.
    Unknown(String),
}

impl FileFormat {
    /// True if this is a recognized format (not `Unknown`).
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    /// Canonical uppercase representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pdf => "PDF",
            Self::Epub => "EPUB",
            Self::Mobi => "MOBI",
            Self::Azw3 => "AZW3",
            Self::Djvu => "DJVU",
            Self::Cbr => "CBR",
            Self::Cbz => "CBZ",
            Self::Fb2 => "FB2",
            Self::Txt => "TXT",
            Self::Doc => "DOC",
            Self::Docx => "DOCX",
            Self::Rtf => "RTF",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for FileFormat {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "epub" => Self::Epub,
            "mobi" => Self::Mobi,
            "azw3" => Self::Azw3,
            "djvu" => Self::Djvu,
            "cbr" => Self::Cbr,
            "cbz" => Self::Cbz,
            "fb2" => Self::Fb2,
            "txt" => Self::Txt,
            "doc" => Self::Doc,
            "docx" => Self::Docx,
            "rtf" => Self::Rtf,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Content type ─────────────────────────────────────────────────

/// Known content categories.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    Book,
    Paper,
    Magazine,
    Standards,
    Comic,
    /// Category not in the known set.
    Unknown(String),
}

impl From<&str> for ContentType {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "book_fiction" | "book_nonfiction" | "book_unknown" | "book" => {
                Self::Book
            }
            "journal_article" | "paper" => Self::Paper,
            "magazine" => Self::Magazine,
            "standards_document" | "standards" => Self::Standards,
            "comic" => Self::Comic,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Book => f.write_str("book"),
            Self::Paper => f.write_str("paper"),
            Self::Magazine => f.write_str("magazine"),
            Self::Standards => f.write_str("standards"),
            Self::Comic => f.write_str("comic"),
            Self::Unknown(s) => f.write_str(s),
        }
    }
}

// ── Md5 ──────────────────────────────────────────────────────────

/// An MD5 content hash identifying an item in Anna's Archive.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Md5([u8; 16]);

impl Md5 {
    /// Construct from raw bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// The raw 16-byte hash.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Hex string representation.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl std::fmt::Debug for Md5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Md5({})", self.to_hex())
    }
}

impl std::fmt::Display for Md5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl From<&str> for Md5 {
    fn from(s: &str) -> Self {
        let mut bytes = [0u8; 16];
        // Parse hex string — if malformed, store as zero-padded.
        for (i, chunk) in s.as_bytes().chunks(2).take(16).enumerate() {
            let hex = std::str::from_utf8(chunk).unwrap_or("00");
            bytes[i] = u8::from_str_radix(hex, 16).unwrap_or(0);
        }
        Self(bytes)
    }
}

impl From<String> for Md5 {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl Serialize for Md5 {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Md5 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Self::from(s.as_str()))
    }
}

// ── Search types ─────────────────────────────────────────────────

/// A single search result from the HTML search page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub md5: Md5,
    pub title: String,
    pub author: Option<String>,
    pub format: Option<FileFormat>,
    pub size: Option<String>,
    pub language: Option<String>,
}

/// Options for a search query.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub page: Option<u32>,
}

impl SearchOptions {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            page: None,
        }
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }
}

/// Search response containing results and pagination info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub page: u32,
    pub has_more: bool,
}

/// Metadata extracted from the "format · size · language" line.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub format: Option<FileFormat>,
    pub size: Option<String>,
    pub language: Option<String>,
}

// ── Download types ───────────────────────────────────────────────

/// Request parameters for resolving a download URL.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub md5: Md5,
    pub path_index: Option<u32>,
    pub domain_index: Option<u32>,
}

impl DownloadRequest {
    pub fn new(md5: impl Into<Md5>) -> Self {
        Self {
            md5: md5.into(),
            path_index: None,
            domain_index: None,
        }
    }
}

/// Download URL returned by the fast download API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub download_url: String,
}

// ── Item detail types ────────────────────────────────────────────

/// Identifiers for an item (ISBN, DOI, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Identifiers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn10: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asin: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blake2b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_library: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_books: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goodreads: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amazon: Option<Vec<String>>,
}

impl Identifiers {
    /// Returns true if at least one identifier field is populated.
    pub fn has_any(&self) -> bool {
        self.isbn10.is_some()
            || self.isbn13.is_some()
            || self.doi.is_some()
            || self.asin.is_some()
            || self.sha1.is_some()
            || self.sha256.is_some()
            || self.crc32.is_some()
            || self.blake2b.is_some()
            || self.open_library.is_some()
            || self.google_books.is_some()
            || self.goodreads.is_some()
            || self.amazon.is_some()
    }
}

/// IPFS availability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsInfo {
    pub cid: String,
    pub from: String,
}

/// A download mirror.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSource {
    pub name: String,
    pub url: String,
}

/// Full item details from the JSON API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDetails {
    pub md5: Md5,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FileFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifiers: Option<Identifiers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subjects: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cids: Option<Vec<IpfsInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_sources: Option<Vec<DownloadSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_paths: Option<Vec<String>>,
}

// ── Client config (init envelope) ────────────────────────────────

/// Init envelope for [`Client`](crate::Client).
#[derive(Debug, Clone)]
pub struct Config {
    pub domains: Vec<String>,
    pub user_agent: String,
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            domains: vec![
                "annas-archive.gd".into(),
                "annas-archive.gs".into(),
            ],
            user_agent: format!("annas-archive/{}", env!("CARGO_PKG_VERSION")),
            api_key: None,
        }
    }
}
