use actix_web::{post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;

#[derive(Deserialize)]
struct SearchRequest {
    word: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<String>,
}

async fn search_elasticsearch(word: String) -> Result<Vec<String>, Box<dyn Error>> {
    dotenv().ok();
    let elasticsearch_url = env::var("ELASTICSEARCH_URL")?;

    let transport = Transport::single_node(&elasticsearch_url)?;
    let client = Elasticsearch::new(transport);

    let query = json!({
        "query": {
            "match": {
                "word": {
                    "query": word,
                    "fuzziness": "AUTO"
                }
            }
        }
    });

    let response = client
        .search(SearchParts::Index(&["autocomplete_index"]))
        .body(query)
        .send()
        .await?;

    let response_body = response.json::<Value>().await?;
    let hits = response_body["hits"]["hits"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|hit| hit["_source"]["word"].as_str().unwrap_or("").to_string())
        .collect::<Vec<String>>();

    Ok(hits)
}

#[post("/search")]
async fn search(req_body: web::Json<SearchRequest>) -> impl Responder {
    match search_elasticsearch(req_body.word.clone()).await {
        Ok(results) => web::Json(SearchResponse { results }),
        Err(_) => web::Json(SearchResponse { results: vec![] }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| App::new().route("/", web::get().to(|| async { "Rust App Home" })))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
