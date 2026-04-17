# VectorDB RAG Demo

A fully containerized Document RAG demo with a dark-themed web UI. Upload documents, chat with your knowledge base.

## Quick Start

```bash
# 1. Fill in your API key
cp .env.example .env
# Edit .env and set OPENAI_API_KEY=sk-your-key-here

# 2. Build and start everything
docker compose up -d --build

# 3. Open browser
open http://localhost:3035
```

That's it! No other setup required.

## Features

- **File Upload** - Drag & drop or browse documents (PDF, TXT, MD, DOCX, CSV)
- **Chat Interface** - Ask questions, get AI-powered answers
- **Fully Containerized** - VortexDB + Backend + Frontend in Docker

## Architecture

```
Browser (localhost:3035) → Frontend (nginx)
                              ↓
                     Backend API (port 8000)
                              ↓
              ┌───────────────┴───────────────┐
              ↓                               ↓
        OpenAI API                    VortexDB
        (embeddings + LLM)            (HTTP port 3000)
```

## Configuration

### .env file

```env
OPENAI_API_KEY=sk-your-api-key-here
VORTEXDB_HOST=vortexdb
VORTEXDB_PORT=3000
EMBEDDING_MODEL=text-embedding-3-small
LLM_MODEL=gpt-4o-mini
CHUNK_SIZE=512
CHUNK_OVERLAP=50
TOP_K=5
```

## Project Structure

```
demo/document-rag/
├── docker-compose.yml
├── .env.example
├── README.md
├── backend/
│   ├── src/
│   │   ├── main.py        # FastAPI app
│   │   ├── config.py      # Config from env
│   │   ├── chunker.py     # Text chunking
│   │   ├── embedder.py    # OpenAI embeddings
│   │   ├── generator.py   # Chat completion
│   │   ├── extractor.py  # Document parsing
│   │   └── vectorstore.py # VortexDB HTTP client
│   ├── requirements.txt
│   └── Dockerfile
└── frontend/
    ├── src/
    │   ├── App.jsx        # Main React component
    │   ├── App.css        # Styles
    │   └── main.jsx       # Entry point
    ├── index.html
    ├── package.json
    ├── vite.config.js
    ├── nginx.conf
    └── Dockerfile
```
