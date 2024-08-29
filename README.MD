# Elasticsearch Rust Autocomplete Setup

This guide provides steps to set up Elasticsearch with autocomplete functionality that handles typos, using a Rust application for interaction.

## Setup and Run

1. Start Services with Docker Compose

`docker-compose up -d`

2. Add Index Configuration for Elasticsearch

```
curl -X PUT "http://localhost:9200/autocomplete_index" -H 'Content-Type: application/json' -d'
{
    "settings": {
        "analysis": {
            "filter": {
                "autocomplete_filter": {
                    "type": "edge_ngram",
                    "min_gram": 1,
                    "max_gram": 20
                }
            },
            "analyzer": {
                "autocomplete_analyzer": {
                    "type": "custom",
                    "tokenizer": "standard",
                    "filter": [
                        "lowercase",
                        "autocomplete_filter"
                    ]
                },
                "autocomplete_search_analyzer": {
                    "type": "custom",
                    "tokenizer": "standard",
                    "filter": [
                        "lowercase"
                    ]
                }
            }
        }
    },
    "mappings": {
        "properties": {
            "word": {
                "type": "text",
                "analyzer": "autocomplete_analyzer",
                "search_analyzer": "autocomplete_search_analyzer"
            }
        }
    }
}
'
```

3. Make Bulk Indexing Script Executable

`chmod +x setup-elasticsearch/bulk_index.sh`

4. Run Indexing of Words

`./setup-elasticsearch/bulk_index.sh`

5. Test the Setup

- Test with Typo "appple"

```
curl -X POST http://localhost:8080/search \
-H "Content-Type: application/json" \
-d '{"word": "appple"}'
```

Expected output:

`{"results":["apoplectic","apoplectical","apoplectically","apoplectiform","apoplectoid","apoplex","apoplexy","apoplexies","apoplexious","appleberry"]}`

- Test with "appledrone"

```
curl -X POST http://localhost:8080/search \
-H "Content-Type: application/json" \
-d '{"word": "appledrone"}'
```

Expected output:

`{"results":["appledrone","appledrane","applegrower"]}`
