//! Live integration test — requires network access.
//! Run with: cargo test --test live_search -- --ignored

use annas_archive::{Client, SearchOptions};

#[tokio::test]
#[ignore] // requires network
async fn live_search() {
    let client = Client::new();
    let response = client
        .search(SearchOptions::new("category theory"))
        .await
        .expect("search should succeed");

    println!("page: {}", response.page);
    println!("has_more: {}", response.has_more);
    println!("results: {}", response.results.len());

    for r in &response.results {
        println!(
            "  [{md5}] {title} — {author} ({fmt}, {size})",
            md5 = &r.md5[..8],
            title = r.title,
            author = r.author.as_deref().unwrap_or("?"),
            fmt = r
                .format
                .as_ref()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            size = r.size.as_deref().unwrap_or("?"),
        );
    }

    assert!(!response.results.is_empty(), "should find results");
    assert_eq!(response.page, 1);

    // Verify structured types
    let first = &response.results[0];
    assert!(!first.md5.is_empty());
    assert!(!first.title.is_empty());
}
