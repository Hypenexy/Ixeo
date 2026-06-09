use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value}; // <-- Explicitly imported the Value trait here
use tantivy::{Index, IndexReader, TantivyDocument};
use tower_http::cors::{Any, CorsLayer};

// Shared state context for Axum threads
struct AppState {
    index: Index,
    reader: IndexReader,
    schema: Schema,
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

#[derive(Serialize)]
struct SearchResult {
    url: String,
    title: String,
    score: f32,
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

    let shared_state = Arc::new(AppState { index, reader, schema });

    // Enable cross-origin calls so your Javascript UI layer can fetch data securely
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any);

    let app = Router::new()
        .route("/search", get(handle_search))
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

    // Get handle fields from the instantiated schema
    let title_field = state.schema.get_field("title").unwrap();
    let body_field = state.schema.get_field("body").unwrap();
    let url_field = state.schema.get_field("url").unwrap();

    // Configure query parser to examine both headings/titles and paragraphs
    let query_parser = QueryParser::for_index(&state.index, vec![title_field, body_field]);
    
    let query = match query_parser.parse_query(query_str) {
        Ok(q) => q,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid search syntax").into_response(),
    };

    let searcher = state.reader.searcher();
    
    // Execute search tracking top 20 relevant results using .order_by_score()
    let top_docs = match searcher.search(&query, &TopDocs::with_limit(20).order_by_score()) {
        Ok(docs) => docs,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut results = Vec::new();

    for (score, doc_address) in top_docs {
        if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
            // Using the Value trait's .as_str() method
            let url = retrieved_doc.get_first(url_field).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let title = retrieved_doc.get_first(title_field).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            
            results.push(SearchResult { url, title, score });
        }
    }

    (StatusCode::OK, Json(results)).into_response()
}