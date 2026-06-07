use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use dotenvy::dotenv;
use std::env;

// This macro initializes Tokio, giving us an async main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load the .env file in development
    dotenv().ok(); 

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url)?;
    // ... rest of the code
    println!("Initializing little evil Spider Node...");

    // 1. Initialize the HTTP Client
    // We use a single client pool for connection reuse (Keep-Alive)
    let http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("IxeoCrawler/1.0 (+https://ixeo.midelight.net/bot)")
        .build()?;

    // 2. Connect to the URL Frontier (Redis)
    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let mut redis_conn = redis_client.get_async_connection().await?;
    println!("Connected to URL Frontier.");

    // 3. Connect to the Document Store (PostgreSQL via SQLx)
    // let db_pool = sqlx::postgres::PgPoolOptions::new()
    //     .max_connections(50)
    //     .connect("postgres://user:pass@localhost/search_db").await?;

    // Start crawling
    // In reality, this would be a loop spawning tokio::spawn workers
    let seed_url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    fetch_and_parse(&http_client, seed_url).await?;

    Ok(())
}

async fn fetch_and_parse(client: &Client, target_url: &str) -> Result<()> {
    println!("Fetching: {}", target_url);
    
    // Fetch the page
    let response = client.get(target_url).send().await?;
    
    // Ensure we got a 200 OK
    if response.status().is_success() {
        let html_content = response.text().await?;
        println!("Successfully fetched {} bytes.", html_content.len());
        
        // Next steps:
        // 1. Pass `html_content` to the `scraper` crate to extract text.
        // 2. Extract all <a href="..."> tags.
        // 3. Push new URLs to Redis.
        // 4. Push raw text to PostgreSQL.
    } else {
        println!("Failed to fetch. Status: {}", response.status());
    }

    Ok(())
}