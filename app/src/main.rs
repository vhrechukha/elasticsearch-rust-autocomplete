use actix_web::HttpRequest;
use actix_web::{post, web, App, HttpServer, Responder};
use awc::Client;
use dotenv::dotenv;
use elasticsearch::{http::transport::Transport, Elasticsearch, SearchParts};
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

async fn forward_to_elasticsearch(req: HttpRequest) -> impl Responder {
    let client = Client::default();
    let elastic_url = format!("http://elasticsearch:9200{}", req.uri());
    let mut response = client.get(elastic_url).send().await.unwrap();
    let body = response.body().await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn forward_to_kibana(req: HttpRequest) -> impl Responder {
    let client = Client::default();
    let kibana_url = format!("http://kibana:5601{}", req.uri());
    let mut response = client.get(kibana_url).send().await.unwrap();
    let body = response.body().await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
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
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(|| async { "Rust App Home" }))
            .route(
                "/elasticsearch/{tail:.*}",
                web::to(forward_to_elasticsearch),
            )
            .route("/kibana/{tail:.*}", web::to(forward_to_kibana))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
