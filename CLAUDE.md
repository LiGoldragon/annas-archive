# CLAUDE.md

## Overview

Rust client library for Anna's Archive — a meta-index aggregating LibGen,
Sci-Hub, Z-Library, Internet Archive, and other shadow libraries. Leaf
crate in the Mentci ecosystem, no dependencies on ecosystem crates.

## API surface

| Method | API key? | What it does |
|--------|----------|--------------|
| `Client::search(SearchOptions)` | No | HTML scrape of search results |
| `Client::details(md5)` | Yes | JSON API for rich item metadata |
| `Client::download_url(DownloadRequest)` | Yes | Fast download URL resolution |

## Domain failover

Domains change frequently due to legal pressure. Current defaults:
`annas-archive.gd`, `annas-archive.gs`. Override via `Config::domains`.

## Testing

```
cargo test                                        # unit tests (offline)
cargo test --test live_search -- --ignored        # live network test
```

## Conventions

Follows Mentci RUST_PATTERNS.md:
- `Error` not `AnnaError` (crate scope provides namespace)
- `impl From<&str>` not `from_str_loose()` (trait-domain rule)
- `SearchResponse::from_html()` not `parse_search_results()` (everything is an object)
- `DownloadRequest` not `(md5, path_index, domain_index)` (single object in/out)
- Structured error variants, no `context: String` bags

## VCS

Jujutsu (`jj`) is mandatory. Git is the backend only.
