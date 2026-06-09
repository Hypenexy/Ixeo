use anyhow::Result;
use dotenvy::dotenv;
use scraper::{Html, Selector};
use sqlx::PgPool; // Changed from PgPoolOptions for simplicity
use std::env;
use std::path::Path;
use std::time::Duration;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter};

// Define a struct to map our Postgres database rows at runtime
#[derive(sqlx::FromRow)]
struct RawPage {
    id: i32,
    url: String,
    title: Option<String>,
    html_content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // 1. Initialize Tantivy Schema
    let mut schema_builder = Schema::builder();
    let url_field = schema_builder.add_text_field("url", TEXT | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let body_field = schema_builder.add_text_field("body", TEXT | STORED);
    let schema = schema_builder.build();

    let index_path = Path::new("./tantivy_index");
    std::fs::create_dir_all(index_path)?;

    let index = match Index::open_in_dir(index_path) {
        Ok(idx) => idx,
        Err(_) => Index::create_in_dir(index_path, schema)?,
    };

    let mut index_writer = index.writer(50_000_000)?;

    // 2. Connect to PostgreSQL
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&db_url).await?;

    // Ensure our status column exists
    sqlx::query("ALTER TABLE raw_pages ADD COLUMN IF NOT EXISTS indexed BOOLEAN DEFAULT FALSE;")
        .execute(&db_pool)
        .await?;

    println!("Indexer worker online. Watching PostgreSQL for unindexed pages...");

    // 3. The Continuous Processing Loop
    loop {
        // Switched to query_as (Runtime check) instead of query_as! (Compile-time check)
        let rows = sqlx::query_as::<_, RawPage>(
            "SELECT id, url, title, html_content FROM raw_pages WHERE indexed = FALSE LIMIT 50"
        )
        .fetch_all(&db_pool)
        .await?;

        if rows.is_empty() {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        println!("Processing batch of {} pages...", rows.len());

        for row in rows {
            let raw_html = row.html_content.unwrap_or_default();
            let clean_body = extract_text_content(&raw_html);
            let page_title = row.title.unwrap_or_else(|| "Untitled".to_string());
            let page_url = row.url;

            index_writer.add_document(doc!(
                url_field => page_url,
                title_field => page_title,
                body_field => clean_body
            ))?;

            // Switched to basic query (Runtime check)
            sqlx::query("UPDATE raw_pages SET indexed = TRUE WHERE id = $1")
                .bind(row.id)
                .execute(&db_pool)
                .await?;
        }

        index_writer.commit()?;
        println!("Batch committed to Tantivy index successfully.");
    }
}

fn extract_text_content(html: &str) -> String {
    let mut document = Html::parse_document(html);
    let junk_tags = ["script", "style", "nav", "footer", "header", "aside", "noscript"];

    for tag in junk_tags {
        let selector = Selector::parse(tag).unwrap();
        let node_ids: Vec<_> = document.select(&selector).map(|n| n.id()).collect();
        for id in node_ids {
            document.tree.get_mut(id).unwrap().detach();
        }
    }

    document
        .root_element()
        .text()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}