#!/bin/bash

bulk_data=""

batch_size=100
counter=0

while IFS= read -r word; do
    bulk_data+="{ \"index\": { \"_index\": \"autocomplete_index\" } }\n{ \"word\": \"$word\" }\n"
    ((counter++))

    if (( counter % batch_size == 0 )); then
        bulk_data+="\n"

        echo -e "Sending bulk request:\n$bulk_data"

        response=$(curl -s -X POST "http://localhost:9200/_bulk" -H 'Content-Type: application/json' --data-binary "$bulk_data")
        
        if [[ $response == *"errors"* ]]; then
            echo "Error during bulk indexing: $response"
        else
            echo "Bulk indexing successful: $response"
        fi

        bulk_data=""
    fi
done < words.txt

if [[ -n "$bulk_data" ]]; then
    bulk_data+="\n"

    echo -e "Sending final bulk request:\n$bulk_data"

    response=$(curl -s -X POST "http://localhost:9200/_bulk" -H 'Content-Type: application/json' --data-binary "$bulk_data")
    
    if [[ $response == *"errors"* ]]; then
        echo "Error during final bulk indexing: $response"
    else
        echo "Final bulk indexing successful: $response"
    fi
fi
