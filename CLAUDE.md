# CLAUDE.md

## Overview

Rust client library for [Anna's Archive](https://annas-archive.gd/) — a
meta-index aggregating LibGen, Sci-Hub, Z-Library, Internet Archive, and
other shadow libraries. This is a leaf crate in the Mentci ecosystem with
no dependencies on ecosystem crates.

## API surface

| Method | API key? | What it does |
|--------|----------|--------------|
| `Client::search(SearchOptions)` | No | HTML scrape of search results |
| `Client::details(md5)` | Yes | JSON API for rich item metadata |
| `Client::download_url(DownloadRequest)` | Yes | Fast download URL resolution |

Search works without authentication. Details and downloads require an
Anna's Archive API key (see below).

## Getting an API key

Anna's Archive grants API access to donors:

1. Go to [annas-archive.gd/donate](https://annas-archive.gd/donate)
2. Donate via cryptocurrency, Amazon gift card, Cash App, or Alipay
   (conventional payment not available due to legal status)
3. After donation, find your **secret key** in Account Settings
4. Pass the key via `Client::with_api_key("your-key")`

Without a key, only `search()` is available. `details()` and
`download_url()` will return `Error::KeyRequired`.

## Domain failover

Anna's Archive domains change frequently due to legal pressure. The
client tries each configured domain in order. Current defaults:
`annas-archive.gd`, `annas-archive.gs`. Override via `Config::domains`.

If all defaults stop working, check the project's wiki or social channels
for current mirrors and pass them via `Config`.

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
