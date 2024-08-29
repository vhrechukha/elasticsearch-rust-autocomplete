use actix_web::{post, web, App, HttpServer, Responder};
use elasticsearch::{Elasticsearch, SearchParts, http::transport::Transport};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;
use dotenv::dotenv;

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
    let server_address = "0.0.0.0:8080";

    HttpServer::new(|| {
        App::new()
            .service(search)
    })
    .bind(server_address)?
    .run()
    .await
}
