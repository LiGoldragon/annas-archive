# annas-archive

Rust client library for [Anna's Archive](https://annas-archive.gd/) — search
and retrieve books, papers, and documents from the world's largest open library
index.

## Usage

```rust
use annas_archive::{Client, SearchOptions};

let client = Client::new();
let response = client.search(SearchOptions::new("category theory")).await?;

for result in &response.results {
    println!("{} — {} ({:?})", result.title,
        result.author.as_deref().unwrap_or("?"),
        result.format);
}
```

## API key

`search()` works without authentication. `details()` and `download_url()`
require an Anna's Archive API key.

Anna's Archive grants API access to donors:

1. Go to [annas-archive.gd/donate](https://annas-archive.gd/donate)
2. Donate via cryptocurrency, Amazon gift card, Cash App, or Alipay
3. Find your **secret key** in Account Settings
4. Pass it to the client:

```rust
let client = Client::with_api_key("your-secret-key");
let details = client.details("md5hash").await?;
let download = client.download_url(DownloadRequest::new("md5hash")).await?;
```

Without a key, these methods return `Error::KeyRequired`.

## Domain failover

Anna's Archive domains change frequently due to legal pressure. The client
tries each configured domain in order. Current defaults: `annas-archive.gd`,
`annas-archive.gs`.

If all defaults stop resolving, find current mirrors and override:

```rust
use annas_archive::{Client, Config};

let client = Client::from_config(Config {
    domains: vec!["annas-archive.newdomain".into()],
    ..Config::default()
});
```

## License

MIT
