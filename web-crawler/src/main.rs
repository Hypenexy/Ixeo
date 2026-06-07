use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use dotenvy::dotenv;
use std::env;
use sqlx::postgres::PgPoolOptions;
use redis::AsyncCommands;

// This macro initializes Tokio, giving us an async main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load the .env file in development,
    // make sure you setup your postgress
    // and use the example if you're running 
    // "docker compose up"
    dotenv().ok();

    // Initialize Clients
    println!("Initializing little evil Spider Node...");
    let http_client = Client::builder()
        .user_agent("IxeoCrawler/1.0 (+https://ixeo.midelight.net/)")
        .build()?;

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env");
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis_conn = redis_client.get_async_connection().await?;
    
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await?;

    println!("Connected to PostgreSQL. Running migrations...");
    sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await?;
    println!("Database schema is up to date!");


    println!("All systems online. Starting crawl...");

    let seed_url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    
    // Pass our database connections into the fetch function
    fetch_and_parse(&http_client, &db_pool, &mut redis_conn, seed_url).await?;

    Ok(())
}

async fn fetch_and_parse(
    client: &Client, 
    db: &sqlx::PgPool, 
    redis_conn: &mut redis::aio::Connection, 
    target_url: &str
) -> Result<()> {
    println!("Fetching: {}", target_url);
    let response = client.get(target_url).send().await?;
    
    if response.status().is_success() {
        let html_content = response.text().await?;
        
        // 1. Parse the HTML
        let document = Html::parse_document(&html_content);
        
        // 2. Extract the Title
        let title_selector = Selector::parse("title").unwrap();
        let title = document.select(&title_selector)
            .next()
            .map(|t| t.inner_html())
            .unwrap_or_else(|| "No Title".to_string());

        println!("Found Page: {}", title);

        // 3. Save to PostgreSQL
        // We use ON CONFLICT DO NOTHING so we don't crash if we crawl the same page twice
        sqlx::query(
            "INSERT INTO raw_pages (url, title, html_content) VALUES ($1, $2, $3) ON CONFLICT (url) DO NOTHING"
        )
        .bind(target_url)
        .bind(&title)
        .bind(&html_content)
        .execute(db)
        .await?;

        println!("Saved to PostgreSQL!");

        // 4. Extract Links and Push to Redis
        let link_selector = Selector::parse("a[href]").unwrap();
        let base_url = Url::parse(target_url)?;
        let mut new_links_count = 0;

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // This converts relative links (like "/wiki/Linux") into absolute URLs
                if let Ok(absolute_url) = base_url.join(href) {
                    let clean_url = absolute_url.as_str().to_string();
                    
                    // Push the link to a Redis Set called "url_frontier"
                    // A Set (SADD) automatically ignores duplicates!
                    let _ : () = redis_conn.sadd("url_frontier", clean_url).await?;
                    new_links_count += 1;
                }
            }
        }
        println!("Pushed {} new links to the Redis URL Frontier.", new_links_count);
    }

    Ok(())
}