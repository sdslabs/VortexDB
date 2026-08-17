from fastapi import FastAPI, Request, UploadFile, File, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager
from typing import Dict
import tempfile
import os

from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded
from src.config import Config
from src.extractor import extract_text, SUPPORTED_EXTENSIONS
from src.chunker import chunk_text
from src.embedder import Embedder
from src.generator import Generator
from src.vectorstore import VectorStore
from pydantic import BaseModel

vector_store: VectorStore = None

embedder: Embedder = None
generator: Generator = None


def get_ip(request: Request) -> str:
    forwared_request_headers = request.headers.get("X-Forwarded-For")
    if forwared_request_headers:
        return forwared_request_headers.split(",")[0].strip()
    else:
        return request.client.host or request.headers.get("X-Real-IP")


limiter = Limiter(key_func=get_ip)


class user_query(BaseModel):
    query: str


@asynccontextmanager
async def lifespan(app: FastAPI):
    global vector_store, embedder, generator

    vector_store = VectorStore(Config.VORTEXDB_HOST, Config.VORTEXDB_PORT)
    try:
        vector_store.health_check()
    except Exception as e:
        raise RuntimeError(
            f"Failed to connect to VortexDB at "
            f"{Config.VORTEXDB_HOST}:{Config.VORTEXDB_PORT}: {e}"
        ) from e

    print(f"Connected to VortexDB at {Config.VORTEXDB_HOST}:{Config.VORTEXDB_PORT}")

    if not Config.OPENAI_API_KEY:
        print("Warning: OPENAI_API_KEY not set.")
    else:
        embedder = Embedder()
        generator = Generator()

    yield


app = FastAPI(title="Document RAG API", lifespan=lifespan)
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=False,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
@limiter.limit("5/minute")
async def root(request: Request):

    return {"status": "ok", "message": "Document RAG API"}


@app.get("/health")
async def health(request: Request):
    if not embedder or not vector_store:
        return JSONResponse(
            status_code=503,
            content={
                "status": "error",
                "message": "Service not ready. Check AI provider config.",
            },
        )
    try:
        vector_store.health_check()
    except Exception as e:
        return JSONResponse(
            status_code=503,
            content={"status": "error", "message": f"VortexDB unreachable: {e}"},
        )
    return {"status": "ok"}


@app.post("/chat")
@limiter.limit("2/minute")
async def chat(request: Request, user_query: user_query):
    global embedder, generator, vector_store

    if not embedder or not generator or not vector_store:
        raise HTTPException(
            status_code=503,
            detail="Service not ready. Configure OpenAI or Cloudflare AI credentials.",
        )

    question = user_query.query
    if len(question) > 10000:
        raise HTTPException(status_code=400, detail="Question is too long")
    if not question:
        raise HTTPException(status_code=400, detail="Question is required")

    query_embedding = embedder.embed_single(question)

    results = vector_store.search(query_embedding, Config.TOP_K)

    answer = generator.generate(question, results)

    return {
        "answer": answer,
        "sources": [
            {
                "text": r["text"][:200] + "..." if len(r["text"]) > 200 else r["text"],
            }
            for r in results
        ],
    }


@app.post("/query")
@limiter.limit("2/minute")
async def query_raw(request: Request, user_query: user_query):
    global embedder, generator, vector_store

    if not embedder or not generator or not vector_store:
        raise HTTPException(
            status_code=503,
            detail="Service not ready. Configure OpenAI or Cloudflare AI credentials.",
        )

    question = user_query.query

    if not question:
        raise HTTPException(status_code=400, detail="Question is required")

    query_embedding = embedder.embed_single(question)

    results = vector_store.search(query_embedding, Config.TOP_K)

    return {
        "sources": [
            {
                "text": r["text"][:200] + "..." if len(r["text"]) > 200 else r["text"],
            }
            for r in results
        ],
    }
