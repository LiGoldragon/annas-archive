use crate::error::Error;
use crate::types::{
    ContentType, DownloadSource, FileFormat, Identifiers, IpfsInfo, ItemDetails,
    Md5,
};

impl ItemDetails {
    /// Construct `ItemDetails` from the Anna's Archive JSON API response.
    ///
    /// The response may be double-encoded (a JSON string containing JSON),
    /// so both forms are handled.
    pub fn from_json(json_str: &str, md5: &Md5) -> Result<Self, Error> {
        let json_str = json_str.trim();

        // Handle double-encoded JSON.
        let decoded =
            if json_str.starts_with('"') && json_str.ends_with('"') {
                serde_json::from_str::<String>(json_str)?
            } else {
                json_str.to_string()
            };

        let data: serde_json::Value = serde_json::from_str(&decoded)?;

        // Check for error response.
        if let Some(error) = data.get("error").and_then(|v| v.as_str()) {
            return Err(Error::Remote {
                message: error.to_string(),
            });
        }

        let file_data =
            data.get("file_unified_data").ok_or(Error::MissingField {
                field: "file_unified_data",
            })?;

        let title = str_field(file_data, "title_best")
            .unwrap_or_else(|| "Unknown".to_string());

        let author = nonempty_str(file_data, "author_best");

        let format = nonempty_str(file_data, "extension_best")
            .map(|s| FileFormat::from(s.as_str()));

        let size_bytes =
            file_data.get("filesize_best").and_then(|v| v.as_u64());

        let size = size_bytes.map(format_filesize);

        let language = file_data
            .get("language_codes")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let publisher = nonempty_str(file_data, "publisher_best");
        let year = nonempty_str(file_data, "year_best");
        let description =
            nonempty_str(file_data, "stripped_description_best");
        let cover_url = nonempty_str(file_data, "cover_url_best");
        let original_filename =
            nonempty_str(file_data, "original_filename_best");
        let added_date = nonempty_str(file_data, "added_date_best");
        let pages = nonempty_str(file_data, "pages_best");
        let edition = nonempty_str(file_data, "edition_varia_best");
        let series = nonempty_str(file_data, "series_best");

        let content_type = nonempty_str(file_data, "content_type_best")
            .map(|s| ContentType::from(s.as_str()));

        let identifiers =
            identifiers_from_json(file_data.get("identifiers_unified"));

        let categories = string_list_from_object(
            file_data.get("classifications_unified"),
        );

        let subjects =
            subjects_from_json(file_data.get("classifications_unified"));

        let ipfs_cids =
            ipfs_infos_from_json(file_data.get("ipfs_infos"));

        let additional = data.get("additional");
        let download_sources = download_sources_from_json(additional);
        let torrent_paths = torrent_paths_from_json(additional);

        Ok(Self {
            md5: md5.clone(),
            title,
            author,
            format,
            size,
            size_bytes,
            language,
            publisher,
            year,
            description,
            cover_url,
            content_type,
            original_filename,
            added_date,
            pages,
            edition,
            series,
            identifiers,
            categories,
            subjects,
            ipfs_cids,
            download_sources,
            torrent_paths,
        })
    }
}

// ── Field extraction helpers ─────────────────────────────────────

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn nonempty_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn str_array(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<Vec<String>> {
    obj.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
    })
}

fn first_str(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from)
}

// ── Identifier parsing ──────────────────────────────────────────

fn identifiers_from_json(
    value: Option<&serde_json::Value>,
) -> Option<Identifiers> {
    let obj = value?.as_object()?;

    let ids = Identifiers {
        isbn10: str_array(obj, "isbn10"),
        isbn13: str_array(obj, "isbn13"),
        doi: str_array(obj, "doi"),
        asin: str_array(obj, "asin"),
        sha1: first_str(obj, "sha1"),
        sha256: first_str(obj, "sha256"),
        crc32: first_str(obj, "crc32"),
        blake2b: first_str(obj, "blake2b"),
        open_library: str_array(obj, "ol"),
        google_books: str_array(obj, "googlebookid"),
        goodreads: str_array(obj, "goodreads"),
        amazon: str_array(obj, "amazon"),
    };

    if ids.has_any() { Some(ids) } else { None }
}

// ── Classification parsing ──────────────────────────────────────

fn string_list_from_object(
    value: Option<&serde_json::Value>,
) -> Option<Vec<String>> {
    let obj = value?.as_object()?;
    let mut result = Vec::new();

    for (key, val) in obj {
        if key == "collection" || key.starts_with('_') {
            continue;
        }
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    let s = s.to_string();
                    if !s.is_empty() && !result.contains(&s) {
                        result.push(s);
                    }
                }
            }
        }
    }

    if result.is_empty() { None } else { Some(result) }
}

fn subjects_from_json(
    classifications: Option<&serde_json::Value>,
) -> Option<Vec<String>> {
    let obj = classifications?.as_object()?;

    if let Some(collection) = obj.get("collection") {
        let list = string_list_from_object(Some(collection));
        if list.is_some() {
            return list;
        }
    }

    for (key, val) in obj {
        if key.contains("subject")
            && let Some(arr) = val.as_array()
        {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !strings.is_empty() {
                return Some(strings);
            }
        }
    }

    None
}

// ── IPFS parsing ─────────────────────────────────────────────────

fn ipfs_infos_from_json(
    value: Option<&serde_json::Value>,
) -> Option<Vec<IpfsInfo>> {
    let arr = value?.as_array()?;
    let infos: Vec<IpfsInfo> = arr
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let cid = obj.get("ipfs_cid")?.as_str()?.to_string();
            let from = obj
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            Some(IpfsInfo { cid, from })
        })
        .collect();

    if infos.is_empty() { None } else { Some(infos) }
}

// ── Download source parsing ──────────────────────────────────────

fn download_sources_from_json(
    additional: Option<&serde_json::Value>,
) -> Option<Vec<DownloadSource>> {
    let obj = additional?.as_object()?;
    let mut sources = Vec::new();

    if let Some(urls) =
        obj.get("download_urls").and_then(|v| v.as_array())
    {
        for url in urls {
            if let Some(url_str) = url.as_str() {
                sources.push(DownloadSource {
                    name: "direct".to_string(),
                    url: url_str.to_string(),
                });
            }
        }
    }

    if let Some(urls) = obj.get("ipfs_urls").and_then(|v| v.as_array()) {
        for url in urls {
            if let Some(url_str) = url.as_str() {
                sources.push(DownloadSource {
                    name: "ipfs".to_string(),
                    url: url_str.to_string(),
                });
            }
        }
    }

    if sources.is_empty() { None } else { Some(sources) }
}

fn torrent_paths_from_json(
    additional: Option<&serde_json::Value>,
) -> Option<Vec<String>> {
    let arr = additional?
        .as_object()?
        .get("torrent_paths")?
        .as_array()?;

    let paths: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if paths.is_empty() { None } else { Some(paths) }
}

// ── Formatting ───────────────────────────────────────────────────

fn format_filesize(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_filesize() {
        assert_eq!(format_filesize(0), "0B");
        assert_eq!(format_filesize(512), "512B");
        assert_eq!(format_filesize(1024), "1.0KB");
        assert_eq!(format_filesize(1024 * 1024), "1.0MB");
        assert_eq!(format_filesize(1024 * 1024 * 1024), "1.0GB");
        assert_eq!(format_filesize(6_700_000), "6.4MB");
    }

    #[test]
    fn test_from_json_minimal() {
        let json = r#"{
            "file_unified_data": {
                "title_best": "Test Book",
                "author_best": "Jane Doe",
                "extension_best": "pdf",
                "filesize_best": 1048576,
                "language_codes": ["en"]
            }
        }"#;

        let details =
            ItemDetails::from_json(json, &Md5::from("abc123")).expect("should parse");
        assert_eq!(details.title, "Test Book");
        assert_eq!(details.author.as_deref(), Some("Jane Doe"));
        assert_eq!(details.format, Some(FileFormat::Pdf));
        assert_eq!(details.size.as_deref(), Some("1.0MB"));
        assert_eq!(details.size_bytes, Some(1048576));
        assert_eq!(details.language.as_deref(), Some("en"));
        assert_eq!(details.md5, Md5::from("abc123"));
    }

    #[test]
    fn test_from_json_with_identifiers() {
        let json = r#"{
            "file_unified_data": {
                "title_best": "ISBN Book",
                "identifiers_unified": {
                    "isbn13": ["978-0-13-468599-1"],
                    "doi": ["10.1234/test"]
                }
            }
        }"#;

        let details =
            ItemDetails::from_json(json, &Md5::from("def456")).expect("should parse");
        let ids = details.identifiers.expect("should have identifiers");
        assert_eq!(
            ids.isbn13.as_deref(),
            Some(&["978-0-13-468599-1".to_string()][..])
        );
        assert_eq!(
            ids.doi.as_deref(),
            Some(&["10.1234/test".to_string()][..])
        );
    }

    #[test]
    fn test_from_json_error_response() {
        let json = r#"{"error": "not found"}"#;
        let err = ItemDetails::from_json(json, &Md5::from("bad0")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_from_json_double_encoded() {
        let inner = r#"{"file_unified_data":{"title_best":"Double"}}"#;
        let outer = serde_json::to_string(inner).unwrap();
        let details =
            ItemDetails::from_json(&outer, &Md5::from("ddb1")).expect("should parse");
        assert_eq!(details.title, "Double");
    }

    #[test]
    fn test_content_type_from_str() {
        assert_eq!(ContentType::from("book_fiction"), ContentType::Book);
        assert_eq!(
            ContentType::from("journal_article"),
            ContentType::Paper
        );
        assert_eq!(
            ContentType::from("whatever"),
            ContentType::Unknown("whatever".into())
        );
    }
}
