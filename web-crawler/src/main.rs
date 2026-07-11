use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use dotenvy::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sqlx::postgres::PgPoolOptions;
use redis::AsyncCommands;

const MAX_FRONTIER_SIZE: usize = 2_000;
const VISITED_TTL_SECS: i64 = 60 * 60 * 24;
const DOMAIN_TTL_SECS: i64 = 60 * 60 * 6;

async fn mark_seen(redis_conn: &mut redis::aio::Connection, url: &str) -> Result<bool> {
    let seen_key = format!("seen:{}", url);
    let is_new: bool = redis_conn.set_nx(&seen_key, "1").await?;

    if is_new {
        let _: () = redis_conn.expire(&seen_key, VISITED_TTL_SECS).await?;
    }

    Ok(is_new)
}

async fn is_seen(redis_conn: &mut redis::aio::Connection, url: &str) -> Result<bool> {
    let seen_key = format!("seen:{}", url);
    let exists: bool = redis_conn.exists(&seen_key).await?;
    Ok(exists)
}

async fn enqueue_url(redis_conn: &mut redis::aio::Connection, url: &str) -> Result<bool> {
    let frontier_size: usize = redis_conn.scard("url_frontier").await?;
    if frontier_size >= MAX_FRONTIER_SIZE {
        return Ok(false);
    }

    let inserted: bool = redis_conn.sadd("url_frontier", url).await?;
    if inserted {
        let _: () = redis_conn.expire("url_frontier", 60 * 60 * 6).await?;
    }

    Ok(inserted)
}

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
        .timeout(Duration::from_secs(5))
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
    let _ : () = redis_conn.sadd("url_frontier", seed_url).await?;
    
    loop {
        // 1. Pop a random URL from the frontier
        // SPOP removes it from 'url_frontier' and gives it to us
        let target_url: Option<String> = redis_conn.spop("url_frontier").await?;

        match target_url {
            Some(url) => {
                // 2. THE LOOP BREAKER
                // Try to mark this URL as seen. If it was already seen recently, is_new will be false.
                let is_new = mark_seen(&mut redis_conn, &url).await?;

                if !is_new {
                    println!("Skipping already visited: {}", url);
                    continue; // Skip the rest of the loop and grab the next URL
                }

                // 3. Crawl the URL
                // We wrap this in a match/if-let so that if a single page fails (e.g. 404 error), 
                // it doesn't crash our entire crawler. It just prints the error and moves on.
                if let Err(e) = fetch_and_parse(&http_client, &db_pool, &mut redis_conn, &url).await {
                    eprintln!("Error crawling {}: {}", url, e);
                }

                // 4. THE POLITENESS DELAY
                // Never hit servers instantly back-to-back. 
                // A 500ms delay ensures we are a "good bot".
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            None => {
                // If the queue is entirely empty, wait 5 seconds before checking again
                println!("URL Frontier is empty! Waiting for new links...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn fetch_and_parse(
    client: &Client, 
    db: &sqlx::PgPool, 
    redis_conn: &mut redis::aio::Connection, 
    target_url: &str
) -> Result<()> {
    println!("Fetching: {}", target_url);

    let parsed_url = Url::parse(target_url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL missing host: {}", target_url))?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let last_seen: Option<u64> = redis_conn.hget("domain_last_visited", host).await?;

    if let Some(previous_timestamp) = last_seen {
        let elapsed = now.saturating_sub(previous_timestamp);
        if elapsed < 2 {
            let wait_secs = 2 - elapsed;
            let wait_duration = Duration::from_secs(wait_secs);
            println!("Politeness delay for {}: waiting {:?}", host, wait_duration);
            tokio::time::sleep(wait_duration).await;
        }
    }

    let _ : () = redis_conn.hset("domain_last_visited", host, now).await?;
    let _: () = redis_conn.expire("domain_last_visited", DOMAIN_TTL_SECS).await?;
    let response = client.get(target_url).send().await?;
    
    if response.status().is_success() {
        let html_content = response.text().await?;
        let base_url = Url::parse(target_url)?;

        // 1. Parse the HTML
        let document = Html::parse_document(&html_content);
        
        // 2. Extract the Title
        let title_selector = Selector::parse("title").unwrap();
        let title = document.select(&title_selector)
            .next()
            .map(|t| t.inner_html())
            .unwrap_or_else(|| "No Title".to_string());

        let description = find_meta_content(&document, &["description", "og:description", "twitter:description"]);
        let image_data = fetch_image_data(&client, &document, &base_url).await;

        println!("Found Page: {}", title);

        // 3. Save to PostgreSQL
        // We update existing rows so metadata can improve over time.
        sqlx::query(
            "INSERT INTO raw_pages (url, title, description, image_data, html_content) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (url) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description, image_data = EXCLUDED.image_data, html_content = EXCLUDED.html_content"
        )
        .bind(target_url)
        .bind(&title)
        .bind(&description)
        .bind(&image_data)
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
                if let Ok(mut absolute_url) = base_url.join(href) {
                    // Strip URL fragment (#...) to avoid duplicate queueing of the same page.
                    absolute_url.set_fragment(None);
                    let clean_url = absolute_url.as_str().to_string();
                    
                    // PERFORMANCE TWEAK:
                    // Check if we've already seen this link recently before adding it to the queue.
                    let already_visited = is_seen(&mut *redis_conn, &clean_url).await?;

                    if !already_visited {
                        let inserted = enqueue_url(&mut *redis_conn, &clean_url).await?;
                        if inserted {
                            new_links_count += 1;
                        }
                    }
                }
            }
        }
        println!("Pushed {} new links to the Redis URL Frontier.", new_links_count);
    }

    Ok(())
}

fn find_meta_content(document: &Html, names: &[&str]) -> Option<String> {
    let meta_selector = Selector::parse("meta").unwrap();
    for element in document.select(&meta_selector) {
        if let Some(name) = element.value().attr("name").or_else(|| element.value().attr("property")) {
            if names.iter().any(|expected| name.eq_ignore_ascii_case(expected)) {
                if let Some(content) = element.value().attr("content") {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }
    None
}

async fn fetch_image_data(client: &Client, document: &Html, base_url: &Url) -> Option<String> {
    let image_url = find_meta_content(document, &["og:image", "twitter:image", "image_src"])?;
    let resolved_url = resolve_image_url(base_url, &image_url).ok()?;
    let response = client.get(resolved_url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let bytes = response.bytes().await.ok()?;

    let encoded = general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{};base64,{}", content_type, encoded))
}

fn resolve_image_url(base_url: &Url, image_url: &str) -> Result<Url, url::ParseError> {
    Url::parse(image_url).or_else(|_| base_url.join(image_url))
}
