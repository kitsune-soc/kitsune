# kitsune-search

Tailormade search solution for Kitsune based on the tantivy search engine.  
It supports running multiple nodes in parallel using a primary/secondary architecture.

## Configuration

- `INDEX_DIR_PATH`: Path to the directory in which the search indicies are created. If you are in a container, make sure this is a persistent volume
- `LEVENSHTEIN_DISTANCE`: Configures how lenient the search engine is when determining the whether the query is related to the actual text
- `MEMORY_ARENA_SIZE`: Configures a RAM budget for the search engine. If the budget is exhausted, the documents are flushed to disk (has to be at least 3MB)
- `PORT`: Port on which the gRPC server will listen
- `READ_ONLY`: This node is a *read-only* node; It can only search through the index.
