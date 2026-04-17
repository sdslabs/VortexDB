from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager
from typing import Dict
import tempfile
import os

from src.config import Config
from src.extractor import extract_text, SUPPORTED_EXTENSIONS
from src.chunker import chunk_text
from src.embedder import Embedder
from src.generator import Generator
from src.vectorstore import VectorStore


vector_store: VectorStore = None
embedder: Embedder = None
generator: Generator = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global vector_store, embedder, generator
    
    if not Config.OPENAI_API_KEY or Config.OPENAI_API_KEY == "sk-your-api-key-here":
        print("Warning: OPENAI_API_KEY not set. Set it in .env file.")
    else:
        embedder = Embedder(Config.OPENAI_API_KEY)
        generator = Generator(Config.OPENAI_API_KEY)
        vector_store = VectorStore(Config.VORTEXDB_HOST, Config.VORTEXDB_PORT)
        print(f"Connected to VortexDB at {Config.VORTEXDB_HOST}:{Config.VORTEXDB_PORT}")
    
    yield


app = FastAPI(title="Document RAG API", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
async def root():
    return {"status": "ok", "message": "Document RAG API"}


@app.get("/health")
async def health():
    if not embedder or not vector_store:
        return JSONResponse(
            status_code=503,
            content={"status": "error", "message": "Service not ready. Check API key."}
        )
    return {"status": "ok"}


@app.post("/upload")
async def upload_document(file: UploadFile = File(...)):
    global embedder, vector_store
    
    if not embedder or not vector_store:
        raise HTTPException(status_code=503, detail="Service not ready. Set OPENAI_API_KEY in .env")
    
    ext = os.path.splitext(file.filename)[1].lower()
    if ext not in SUPPORTED_EXTENSIONS:
        raise HTTPException(
            status_code=400,
            detail=f"Unsupported format: {ext}. Supported: {', '.join(SUPPORTED_EXTENSIONS)}"
        )
    
    if ext in ['.png', '.jpg', '.jpeg', '.gif', '.bmp', '.webp']:
        raise HTTPException(
            status_code=400,
            detail="Image files are not supported. Please upload a text document (PDF, TXT, MD, DOCX, CSV)."
        )
    
    with tempfile.NamedTemporaryFile(delete=False, suffix=ext) as tmp:
        content = await file.read()
        tmp.write(content)
        tmp_path = tmp.name
    
    try:
        text = extract_text(tmp_path)
        
        if not text or not text.strip():
            raise HTTPException(status_code=400, detail="Document appears to be empty or no text could be extracted.")
        
        chunks = chunk_text(text, Config.CHUNK_SIZE, Config.CHUNK_OVERLAP)
        
        if not chunks:
            raise HTTPException(status_code=400, detail="Could not chunk document")
        
        embeddings = embedder.embed(chunks)
        
        points_inserted = vector_store.insert_batch(embeddings, chunks, file.filename)
        
        return {
            "success": True,
            "filename": file.filename,
            "chunks": points_inserted,
            "message": f"Document indexed successfully"
        }
    
    except HTTPException:
        raise
    except Exception as e:
        error_msg = str(e)
        if "quota" in error_msg.lower() or "429" in error_msg:
            raise HTTPException(status_code=429, detail="OpenAI API quota exceeded. Please add billing or wait for quota reset.")
        if "clipboard" in error_msg.lower() or "image" in error_msg.lower():
            raise HTTPException(status_code=400, detail="This PDF contains images. Please upload a text-based PDF.")
        raise HTTPException(status_code=500, detail=f"Error processing document: {error_msg}")
    finally:
        os.unlink(tmp_path)


@app.post("/chat")
async def chat(question: str = None, body: Dict = None):
    global embedder, generator, vector_store
    
    if not embedder or not generator or not vector_store:
        raise HTTPException(status_code=503, detail="Service not ready. Set OPENAI_API_KEY in .env")
    
    if body:
        question = body.get("question", question)
    
    if not question:
        raise HTTPException(status_code=400, detail="Question is required")
    
    query_embedding = embedder.embed_single(question)
    
    results = vector_store.search(query_embedding, Config.TOP_K)
    
    answer = generator.generate(question, results)
    
    return {
        "answer": answer,
        "sources": [
            {"text": r["text"][:200] + "..." if len(r["text"]) > 200 else r["text"],
             "filename": r["filename"],
             "score": round(r["score"], 3)}
            for r in results
        ]
    }


@app.delete("/clear")
async def clear():
    global vector_store
    
    if not vector_store:
        raise HTTPException(status_code=503, detail="Service not ready")
    
    vector_store.clear()
    return {"success": True, "message": "All documents cleared"}


@app.get("/stats")
async def stats():
    global vector_store
    
    if not vector_store:
        return {"points_count": 0}
    
    return vector_store.get_info()
