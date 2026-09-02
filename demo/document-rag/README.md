# VortexDB RAG Demo

A containerized document RAG demo: offline ingest into VortexDB, query via a FastAPI backend, and chat through a React frontend. Embeddings and LLM calls use [Cloudflare Workers AI](https://developers.cloudflare.com/workers-ai/) through the OpenAI-compatible API.

## Quick Start

```bash
# 1. Configure credentials
cp .env.example .env
# Set OPENAI_API_KEY (Cloudflare API token) and ACCOUNT_ID

# 2. Add documents to ingest
cp /path/to/your/files/* documents/

# 3. Start the stack (builds images, ingests docs, starts API + UI)
docker compose up -d --build

# 4. Open the UI
open http://localhost:3035
```

On startup, Docker will:

1. Start VortexDB and wait until healthy
2. Run ingest on `./documents` (TXT, MD, PDF, DOCX, CSV)
3. Start the backend API
4. Start the frontend

To re-ingest after adding or changing documents:

```bash
docker compose up ingest --force-recreate
docker compose up -d backend
```

## Ports

| Service   | URL                      |
|-----------|--------------------------|
| Frontend  | http://localhost:3035    |
| Backend   | http://localhost:8000    |
| VortexDB  | http://localhost:3034    |

## Architecture

```
documents/  →  ingest (one-shot)  →  VortexDB
                                         ↑
Browser (3035) → nginx → backend (8000) ─┘
                              ↓
                    Cloudflare Workers AI
                    (embeddings + LLM)
```

## API

| Method | Path     | Body                    | Description              |
|--------|----------|-------------------------|--------------------------|
| GET    | `/health`| —                       | Health check             |
| POST   | `/chat`  | `{"query": "..."}`      | RAG answer + sources     |
| POST   | `/query` | `{"query": "..."}`      | Retrieval only (no LLM)  |

Example:

```bash
curl -X POST http://localhost:8000/chat \
  -H "Content-Type: application/json" \
  -d '{"query": "What embedding model is used?"}'
```

## Configuration

All settings live in `.env`. See `.env.example` for the full list.

```env
OPENAI_API_KEY=your-cloudflare-api-token
ACCOUNT_ID=your-account-id

EMBEDDING_MODEL=@cf/qwen/qwen3-embedding-0.6b
LLM_MODEL=@cf/openai/gpt-oss-20b
VECTOR_SIZE=1024

CHUNK_SIZE=512
CHUNK_OVERLAP=50
TOP_K=5
EMBEDDING_BATCH_SIZE=64
DOCS_DIRECTORY=./documents
```

Get credentials from the [Cloudflare dashboard](https://dash.cloudflare.com/) → Workers AI → **Use REST API**. The API token needs Workers AI read access. `VECTOR_SIZE` must match VortexDB's `DIMENSION` in `docker-compose.yml` (default `1024`).

Inside Docker Compose, `VORTEXDB_HOST` and `VORTEXDB_PORT` are overridden to `vortexdb:3000` automatically.

## Local Development (without Docker)

```bash
# Terminal 1 — VortexDB (from repo root)
HTTP_HOST=0.0.0.0 HTTP_PORT=3034 STORAGE_TYPE=inmemory INDEX_TYPE=flat \
  DIMENSION=1024 cargo run --release --bin server

# Terminal 2 — ingest
cd backend
pip install -r requirements.txt
cp ../.env.example ../.env   # fill in credentials
python -m src.ingestion

# Terminal 3 — API
uvicorn src.main:app --reload --port 8000
```

## Project Structure

```
demo/document-rag/
├── docker-compose.yml
├── .env.example
├── documents/              # Drop files here for ingest
├── backend/
│   ├── src/
│   │   ├── main.py         # FastAPI app
│   │   ├── ingestion.py    # Offline ingest CLI
│   │   ├── config.py       # Env-based config
│   │   ├── chunker.py
│   │   ├── embedder.py     # Cloudflare embeddings
│   │   ├── generator.py    # Cloudflare chat completions
│   │   ├── extractor.py
│   │   └── vectorstore.py  # VortexDB HTTP client
│   ├── requirements.txt
│   └── Dockerfile
└── frontend/
    ├── src/App.jsx
    ├── nginx.conf          # Proxies /api → backend:8000
    └── Dockerfile
```
