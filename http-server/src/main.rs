use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json,
    Router,
};
use axum::http::header;
use chrono::Utc;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use scraper::{Html as ScraperHtml, Selector};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value};
use tantivy::{Index, IndexReader, TantivyDocument};
use tower_http::cors::{Any, CorsLayer};
use url::Url;
// use tower_http::services::ServeDir;

// Shared state context for Axum threads
struct AppState {
    index: Index,
    reader: IndexReader,
    schema: Schema,
    db_pool: PgPool,
    http_client: reqwest::Client,
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    p: Option<usize>,
    url: Option<String>,
}

#[derive(Serialize)]
struct SearchResult {
    url: String,
    title: String,
    description: Option<String>,
    image_data: Option<String>,
    score: f32,
}

#[derive(sqlx::FromRow, Serialize)]
struct CachedMetadata {
    url: String,
    title: String,
    description: Option<String>,
    image_url: Option<String>,
    content_type: Option<String>,
    fetched_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
struct MetadataResponse {
    url: String,
    title: String,
    description: Option<String>,
    image_url: Option<String>,
    content_type: Option<String>,
    fetched_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let index_path = Path::new("../tantivy_index");

    // Open the index created by our indexer process
    let index = Index::open_in_dir(index_path)
        .expect("Failed to open Tantivy index. Is the indexer running and initialized?");

    let reader = index.reader()?;
    let schema = index.schema();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("IxeoMetadataFetcher/1.0")
        .build()?;

    let shared_state = Arc::new(AppState {
        index,
        reader,
        schema,
        db_pool,
        http_client,
    });

    // Enable cross-origin calls so your Javascript UI layer can fetch data securely
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any);

    let app = Router::new()
        .route("/", get(|| async { 
            Html(include_str!("../dist/index.html")) 
        }))
        .route("/api/index.html", get(|| async {
            Html(include_str!("../../html-client/api_docs.html"))
        }))
        .route("/search", get(|| async { 
            Html(include_str!("../dist/search.html")) 
        }))
        .route("/script.js", get(|| async {
            (
                [
                    (header::CONTENT_TYPE, "text/javascript"),
                    (header::CACHE_CONTROL, "public, max-age=86400")
                ], 
                include_str!("../dist/script.js")
            ) 
        }))
        .route("/style.css", get(|| async {
            (
                [
                    (header::CONTENT_TYPE, "text/css"),
                    (header::CACHE_CONTROL, "public, max-age=86400")
                ], 
                include_str!("../dist/style.css")
            ) 
        }))
        .route("/search.css", get(|| async {
            (
                [
                    (header::CONTENT_TYPE, "text/css"),
                    (header::CACHE_CONTROL, "public, max-age=86400")
                ], 
                include_str!("../dist/search.css")
            ) 
        }))
        .route("/api/metadata", get(handle_metadata))
        .route("/api/search", get(handle_search))
        // .fallback_service(ServeDir::new("dist"))
        .layer(cors)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Search Engine HTTP Server running on http://localhost:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_search(
    Query(params): Query<SearchParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let query_str = match params.q {
        Some(ref q) if !q.trim().is_empty() => q,
        _ => return (StatusCode::BAD_REQUEST, "Missing query parameter 'q'").into_response(),
    };

    let page = params.p.unwrap_or(1).max(1);
    let results_per_page = 20; 
    let offset = (page - 1) * results_per_page;

    // Get handle fields from the instantiated schema
    let title_field = state.schema.get_field("title").unwrap();
    let body_field = state.schema.get_field("body").unwrap();
    let url_field = state.schema.get_field("url").unwrap();
    let description_field = state.schema.get_field("description");
    let image_data_field = state.schema.get_field("image_data");

    let mut query_fields = vec![title_field, body_field];
    if let Some(field) = description_field {
        query_fields.push(field);
    }

    let query_parser = QueryParser::for_index(&state.index, query_fields);

    let query = match query_parser.parse_query(query_str) {
        Ok(q) => q,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid search syntax").into_response(),
    };

    let searcher = state.reader.searcher();

    // Execute search tracking top 20 relevant results using .order_by_score() with pages now
    let top_docs = match searcher.search(
        &query, 
        &TopDocs::with_limit(results_per_page).and_offset(offset).order_by_score()
    ) {
        Ok(docs) => docs,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let mut results = Vec::new();

    for (score, doc_address) in top_docs {
        if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
            // Using the Value trait's .as_str() method
            let url = retrieved_doc.get_first(url_field).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let title = retrieved_doc.get_first(title_field).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let description = description_field
                .and_then(|field| retrieved_doc.get_first(field))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let image_data = image_data_field
                .and_then(|field| retrieved_doc.get_first(field))
                .and_then(|v| v.as_str())
                .map(str::to_string);

            results.push(SearchResult { url, title, description, image_data, score });
        }
    }

    (StatusCode::OK, Json(results)).into_response()
}

async fn handle_metadata(
    Query(params): Query<SearchParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let target_url = match params.url.as_deref() {
        Some(url) if !url.trim().is_empty() => match normalize_url(url) {
            Ok(normalized) => normalized,
            Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
        },
        _ => return (StatusCode::BAD_REQUEST, "Missing url parameter 'url'").into_response(),
    };

    match fetch_cached_metadata(&state, &target_url).await {
        Ok(metadata) => (StatusCode::OK, Json(metadata)).into_response(),
        Err(err) => (StatusCode::BAD_GATEWAY, err).into_response(),
    }
}

fn normalize_url(raw_url: &str) -> Result<String, String> {
    let parsed = Url::parse(raw_url.trim()).map_err(|_| "Invalid URL provided".to_string())?;
    Ok(parsed.to_string())
}

async fn fetch_cached_metadata(state: &Arc<AppState>, target_url: &str) -> Result<MetadataResponse, String> {
    if let Some(cached) = sqlx::query_as::<_, CachedMetadata>(
        "SELECT url, title, description, image_url, content_type, fetched_at, expires_at FROM url_metadata_cache WHERE url = $1 AND expires_at > NOW() LIMIT 1",
    )
    .bind(target_url)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|err| format!("Database lookup failed: {err}"))?
    {
        return Ok(MetadataResponse {
            url: cached.url,
            title: cached.title,
            description: cached.description,
            image_url: cached.image_url,
            content_type: cached.content_type,
            fetched_at: cached.fetched_at,
            expires_at: cached.expires_at,
        });
    }

    let response = state
        .http_client
        .get(target_url)
        .header(USER_AGENT, "IxeoMetadataFetcher/1.0")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|err| format!("Request failed: {err}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Remote URL returned status {status}"));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    let body = response
        .text()
        .await
        .map_err(|err| format!("Failed to read remote response body: {err}"))?;

    let metadata = extract_metadata(target_url, &body, content_type.as_deref());
    let fetched_at = Utc::now();
    let expires_at = fetched_at + chrono::Duration::days(1);

    sqlx::query(
        "INSERT INTO url_metadata_cache (url, title, description, image_url, content_type, fetched_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (url) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description, image_url = EXCLUDED.image_url, content_type = EXCLUDED.content_type, fetched_at = EXCLUDED.fetched_at, expires_at = EXCLUDED.expires_at",
    )
    .bind(target_url)
    .bind(&metadata.title)
    .bind(&metadata.description)
    .bind(&metadata.image_url)
    .bind(&metadata.content_type)
    .bind(fetched_at)
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .map_err(|err| format!("Failed to cache metadata: {err}"))?;

    Ok(MetadataResponse {
        url: metadata.url,
        title: metadata.title,
        description: metadata.description,
        image_url: metadata.image_url,
        content_type: metadata.content_type,
        fetched_at,
        expires_at,
    })
}

fn extract_metadata(target_url: &str, body: &str, content_type: Option<&str>) -> MetadataResponse {
    if let Some(content_type) = content_type {
        if content_type.to_lowercase().contains("json") {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
                let title = value
                    .get("title")
                    .and_then(|item| item.as_str())
                    .unwrap_or_default()
                    .to_string();
                let description = value
                    .get("description")
                    .and_then(|item| item.as_str())
                    .map(str::to_string);
                let image_url = value
                    .get("image")
                    .and_then(|item| item.as_str())
                    .map(str::to_string);

                return MetadataResponse {
                    url: target_url.to_string(),
                    title: if title.is_empty() {
                        target_url.to_string()
                    } else {
                        title
                    },
                    description,
                    image_url,
                    content_type: Some(content_type.to_string()),
                    fetched_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::days(1),
                };
            }
        }
    }

    let document = ScraperHtml::parse_document(body);
    let title_selector = Selector::parse("title").unwrap();
    let meta_selector = Selector::parse("meta").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .and_then(|element| element.text().next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(target_url)
        .to_string();

    let mut description = None;
    let mut image_url = None;
    let mut og_title = None;

    for element in document.select(&meta_selector) {
        if let Some(name) = element.value().attr("name").or_else(|| element.value().attr("property")) {
            let content = element.value().attr("content").unwrap_or_default();
            match name {
                "description" => description = Some(content.to_string()),
                "og:description" => description = Some(content.to_string()),
                "og:image" => image_url = Some(content.to_string()),
                "og:title" => og_title = Some(content.to_string()),
                _ => {}
            }
        }
    }

    let final_title = og_title.unwrap_or(title);

    MetadataResponse {
        url: target_url.to_string(),
        title: final_title,
        description,
        image_url,
        content_type: content_type.map(str::to_string),
        fetched_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(1),
    }
}