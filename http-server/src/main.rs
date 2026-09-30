use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json,
    Router,
};
use axum::http::{header, HeaderMap, HeaderValue};
use chrono::Utc;
use redis::AsyncCommands;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use scraper::{Html as ScraperHtml, Selector};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use subtle::ConstantTimeEq;
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
    redis_client: Option<redis::Client>,
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

#[derive(sqlx::FromRow, Serialize)]
struct AdminPage {
    url: String,
    title: Option<String>,
    indexed: bool,
    crawled_at: Option<chrono::DateTime<Utc>>,
}

#[derive(sqlx::FromRow, Serialize)]
struct SearchHour {
    hour_bucket: chrono::DateTime<Utc>,
    request_count: i64,
}

#[derive(Serialize)]
struct AdminDashboardResponse {
    generated_at: chrono::DateTime<Utc>,
    database: DatabaseStats,
    crawler: CrawlerStats,
    search: SearchStats,
    recent_pages: Vec<AdminPage>,
    recent_metadata: Vec<CachedMetadata>,
}

#[derive(Serialize)]
struct DatabaseStats {
    pages_total: i64,
    pages_indexed: i64,
    pages_pending: i64,
    pages_last_24h: i64,
    metadata_cache_total: i64,
    metadata_cache_fresh: i64,
}

#[derive(Serialize)]
struct CrawlerStats {
    status: &'static str,
    heartbeat_at: Option<chrono::DateTime<Utc>>,
    frontier_size: Option<usize>,
    pages_crawled_total: Option<i64>,
}

#[derive(Serialize)]
struct SearchStats {
    requests_24h: i64,
    hourly: Vec<SearchHour>,
    access_model: &'static str,
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
    let redis_client = std::env::var("REDIS_URL")
        .ok()
        .and_then(|url| redis::Client::open(url).ok());

    let shared_state = Arc::new(AppState {
        index,
        reader,
        schema,
        db_pool,
        http_client,
        redis_client,
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
        .route("/admin", get(|| async {
            Html(include_str!("../dist/admin.html"))
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
        .route("/admin.js", get(|| async {
            (
                [(header::CONTENT_TYPE, "text/javascript")],
                include_str!("../dist/admin.js")
            )
        }))
        .route("/admin.css", get(|| async {
            (
                [(header::CONTENT_TYPE, "text/css")],
                include_str!("../dist/admin.css")
            )
        }))
        .route("/api/metadata", get(handle_metadata))
        .route("/api/search", get(handle_search))
        .route("/api/admin/stats", get(handle_admin_stats))
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
    let description_field = state.schema.get_field("description").ok();
    let image_data_field = state.schema.get_field("image_data").ok();

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
    let _ = sqlx::query(
        "INSERT INTO search_hourly_metrics (hour_bucket, request_count) VALUES (date_trunc('hour', NOW()), 1) ON CONFLICT (hour_bucket) DO UPDATE SET request_count = search_hourly_metrics.request_count + 1",
    )
    .execute(&state.db_pool)
    .await;
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

async fn handle_admin_stats(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let expected_token = match std::env::var("IXEO_ADMIN_TOKEN") {
        Ok(token) if !token.is_empty() => token,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Admin dashboard is disabled: configure IXEO_ADMIN_TOKEN",
            )
                .into_response();
        }
    };

    let provided_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if !provided_token.is_some_and(|provided| {
        bool::from(expected_token.as_bytes().ct_eq(provided.as_bytes()))
    }) {
        return (StatusCode::UNAUTHORIZED, "Invalid admin token").into_response();
    }

    match load_admin_dashboard(&state).await {
        Ok(stats) => {
            let mut response = Json(stats).into_response();
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not load dashboard data: {error}"),
        )
            .into_response(),
    }
}

async fn load_admin_dashboard(
    state: &Arc<AppState>,
) -> Result<AdminDashboardResponse, sqlx::Error> {
    let (pages_total, pages_indexed, pages_pending, pages_last_24h): (i64, i64, i64, i64) =
        sqlx::query_as(
            "SELECT COUNT(*)::BIGINT, COUNT(*) FILTER (WHERE indexed IS TRUE)::BIGINT, COUNT(*) FILTER (WHERE indexed IS NOT TRUE)::BIGINT, COUNT(*) FILTER (WHERE crawled_at >= NOW() - INTERVAL '24 hours')::BIGINT FROM raw_pages",
        )
        .fetch_one(&state.db_pool)
        .await?;
    let (metadata_cache_total, metadata_cache_fresh): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*)::BIGINT, COUNT(*) FILTER (WHERE expires_at > NOW())::BIGINT FROM url_metadata_cache",
    )
    .fetch_one(&state.db_pool)
    .await?;
    let requests_24h: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(request_count), 0)::BIGINT FROM search_hourly_metrics WHERE hour_bucket >= NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(&state.db_pool)
    .await?;
    let hourly = sqlx::query_as::<_, SearchHour>(
        "SELECT hour_bucket, request_count FROM search_hourly_metrics WHERE hour_bucket >= date_trunc('hour', NOW()) - INTERVAL '23 hours' ORDER BY hour_bucket",
    )
    .fetch_all(&state.db_pool)
    .await?;
    let recent_pages = sqlx::query_as::<_, AdminPage>(
        "SELECT url, title, COALESCE(indexed, FALSE) AS indexed, crawled_at FROM raw_pages ORDER BY crawled_at DESC NULLS LAST LIMIT 20",
    )
    .fetch_all(&state.db_pool)
    .await?;
    let recent_metadata = sqlx::query_as::<_, CachedMetadata>(
        "SELECT url, title, description, image_url, content_type, fetched_at, expires_at FROM url_metadata_cache ORDER BY fetched_at DESC LIMIT 12",
    )
    .fetch_all(&state.db_pool)
    .await?;

    let crawler = if let Some(client) = &state.redis_client {
        match client.get_async_connection().await {
            Ok(mut connection) => {
                let heartbeat: Option<i64> = connection
                    .get("ixeo:crawler:heartbeat")
                    .await
                    .unwrap_or(None);
                let frontier_size: Option<usize> = connection.scard("url_frontier").await.ok();
                let pages_crawled_total: Option<i64> = connection
                    .get("ixeo:crawler:pages_crawled")
                    .await
                    .unwrap_or(None);
                let heartbeat_at = heartbeat
                    .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0));
                let status = match heartbeat_at {
                    Some(timestamp)
                        if Utc::now().signed_duration_since(timestamp).num_seconds() < 60 =>
                    {
                        "online"
                    }
                    Some(_) => "stale",
                    None => "offline",
                };
                CrawlerStats {
                    status,
                    heartbeat_at,
                    frontier_size,
                    pages_crawled_total,
                }
            }
            Err(_) => CrawlerStats {
                status: "unavailable",
                heartbeat_at: None,
                frontier_size: None,
                pages_crawled_total: None,
            },
        }
    } else {
        CrawlerStats {
            status: "not configured",
            heartbeat_at: None,
            frontier_size: None,
            pages_crawled_total: None,
        }
    };

    Ok(AdminDashboardResponse {
        generated_at: Utc::now(),
        database: DatabaseStats {
            pages_total,
            pages_indexed,
            pages_pending,
            pages_last_24h,
            metadata_cache_total,
            metadata_cache_fresh,
        },
        crawler,
        search: SearchStats {
            requests_24h,
            hourly,
            access_model: "anonymous; API tokens and user identities are not configured",
        },
        recent_pages,
        recent_metadata,
    })
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