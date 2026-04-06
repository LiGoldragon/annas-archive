//! Live integration test — search + details with API key.
//! Run with: cargo test --test live_details -- --ignored --nocapture

use bibliotheca::{Client, Md5, SearchOptions};

#[tokio::test]
#[ignore] // requires network + API key
async fn search_and_details() {
    let api_key = std::env::var("ANNAS_ARCHIVE_API_KEY")
        .expect("ANNAS_ARCHIVE_API_KEY must be set");

    let client = Client::with_api_key(api_key);

    // Search for the book
    let response = client
        .search(SearchOptions::new("arthur young geometry of meaning"))
        .await
        .expect("search should succeed");

    println!("results: {}", response.results.len());
    for r in &response.results {
        println!(
            "  [{}] {} — {} ({}, {})",
            &r.md5.to_hex()[..8],
            r.title,
            r.author.as_deref().unwrap_or("?"),
            r.format
                .as_ref()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            r.size.as_deref().unwrap_or("?"),
        );
    }

    assert!(!response.results.is_empty(), "should find results");

    // Get details for the first result
    let first = &response.results[0];
    println!("\nfetching details for {}...", first.md5);

    let details = client
        .details(&first.md5)
        .await
        .expect("details should succeed");

    println!("title: {}", details.title);
    println!("author: {:?}", details.author);
    println!("format: {:?}", details.format);
    println!("size: {:?}", details.size);
    println!("year: {:?}", details.year);
    println!("publisher: {:?}", details.publisher);
    println!("pages: {:?}", details.pages);
    println!("description: {:?}", details.description.as_deref().map(|d| &d[..d.len().min(200)]));
    println!("identifiers: {:?}", details.identifiers);
    println!("ipfs_cids: {:?}", details.ipfs_cids);

    assert_eq!(details.md5, first.md5);
    assert!(!details.title.is_empty());
}
