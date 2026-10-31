import os
import time
import httpx
from fastapi import FastAPI, HTTPException, UploadFile, File
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel
from typing import Optional
import asyncio
from concurrent.futures import ThreadPoolExecutor

from vortexdb import VortexDB, DenseVector, Payload, Similarity

app = FastAPI(title="CLIP Image Search API")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

CLIP_VECTORIZER_URL = os.getenv("CLIP_VECTORIZER_URL", "http://localhost:5000")
VORTEXDB_GRPC_URL = os.getenv("VORTEXDB_GRPC_URL", "localhost:50051")
VORTEXDB_API_KEY = os.getenv("VORTEXDB_API_KEY", "")
IMAGES_DIR = os.getenv("IMAGES_DIR", "./images")

executor = ThreadPoolExecutor(max_workers=10)


def get_vortexdb_client() -> VortexDB:
    """Create a new VortexDB client instance"""
    return VortexDB(
        grpc_url=VORTEXDB_GRPC_URL,
        api_key=VORTEXDB_API_KEY if VORTEXDB_API_KEY else None,
    )

os.makedirs(IMAGES_DIR, exist_ok=True)
app.mount("/images", StaticFiles(directory=IMAGES_DIR), name="images")


class SearchRequest(BaseModel):
    query: str
    limit: Optional[int] = 20


class SearchResult(BaseModel):
    image_url: str
    point_id: str
    score: Optional[float] = None


class SearchResponse(BaseModel):
    results: list[SearchResult]
    query: str
    vectorize_time_ms: float
    search_time_ms: float
    total_time_ms: float


class IndexRequest(BaseModel):
    image_path: str


class IndexResponse(BaseModel):
    point_id: str
    vectorize_time_ms: float
    insert_time_ms: float


class StatsResponse(BaseModel):
    total_images: int
    clip_vectorizer_status: str
    vortexdb_status: str


@app.get("/health")
async def health_check():
    return {"status": "ok"}


@app.get("/stats", response_model=StatsResponse)
async def get_stats():
    """Get system statistics and service health"""
    clip_status = "unknown"
    vortex_status = "unknown"
    
    async with httpx.AsyncClient(timeout=5.0) as client:
        try:
            resp = await client.get(f"{CLIP_VECTORIZER_URL}/docs")
            clip_status = "online" if resp.status_code == 200 else "error"
        except Exception:
            clip_status = "offline"
    
    # Check VortexDB via gRPC client
    def check_vortexdb():
        try:
            db = get_vortexdb_client()
            db.close()
            return "online"
        except Exception:
            return "offline"
    
    loop = asyncio.get_event_loop()
    vortex_status = await loop.run_in_executor(executor, check_vortexdb)
    
    # Count images in directory
    image_count = 0
    if os.path.exists(IMAGES_DIR):
        image_count = len([f for f in os.listdir(IMAGES_DIR) 
                          if f.lower().endswith(('.png', '.jpg', '.jpeg', '.webp', '.gif'))])
    
    return StatsResponse(
        total_images=image_count,
        clip_vectorizer_status=clip_status,
        vortexdb_status=vortex_status
    )


@app.post("/search", response_model=SearchResponse)
async def search_images(request: SearchRequest):
    """
    Search for images similar to the text query.
    Uses CLIP to vectorize text and VortexDB to find similar image vectors.
    """
    total_start = time.perf_counter()
    
    # Vectorize the text query using CLIP
    vectorize_start = time.perf_counter()
    async with httpx.AsyncClient(timeout=30.0) as client:
        try:
            clip_response = await client.post(
                f"{CLIP_VECTORIZER_URL}/vectors",
                json={"text": request.query}
            )
            clip_response.raise_for_status()
            vector = clip_response.json()["result"]
        except httpx.HTTPError as e:
            raise HTTPException(status_code=503, detail=f"CLIP vectorizer error: {str(e)}")
    
    vectorize_time = (time.perf_counter() - vectorize_start) * 1000
    
    # Search VortexDB for similar vectors using the Python client
    def do_search():
        db = get_vortexdb_client()
        try:
            point_ids = db.search(
                vector=DenseVector(vector),
                similarity=Similarity.COSINE,
                limit=request.limit,
            )
            return point_ids
        finally:
            db.close()
    
    search_start = time.perf_counter()
    loop = asyncio.get_event_loop()
    try:
        point_ids = await loop.run_in_executor(executor, do_search)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"VortexDB error: {str(e)}")
    
    search_time = (time.perf_counter() - search_start) * 1000
    
    # Get point details to retrieve image paths
    def get_points_details(point_ids):
        results = []
        db = get_vortexdb_client()
        try:
            for point_id in point_ids:
                try:
                    point = db.get(point_id=point_id)
                    if point and point.payload:
                        image_path = point.payload.content
                        if image_path:
                            filename = os.path.basename(image_path)
                            results.append(SearchResult(
                                image_url=f"/images/{filename}",
                                point_id=str(point_id)
                            ))
                except Exception:
                    continue
            return results
        finally:
            db.close()
    
    results = await loop.run_in_executor(executor, get_points_details, point_ids)
    
    total_time = (time.perf_counter() - total_start) * 1000
    
    return SearchResponse(
        results=results,
        query=request.query,
        vectorize_time_ms=round(vectorize_time, 2),
        search_time_ms=round(search_time, 2),
        total_time_ms=round(total_time, 2)
    )


@app.post("/index", response_model=IndexResponse)
async def index_image(file: UploadFile = File(...)):
    """
    Index a new image: vectorize with CLIP and store in VortexDB
    """
    # Save the uploaded file
    filename = file.filename or f"image_{int(time.time())}.jpg"
    file_path = os.path.join(IMAGES_DIR, filename)
    
    content = await file.read()
    with open(file_path, "wb") as f:
        f.write(content)
    
    # Vectorize the image
    vectorize_start = time.perf_counter()
    async with httpx.AsyncClient(timeout=60.0) as client:
        try:
            files = {"file": (filename, content, file.content_type or "image/jpeg")}
            clip_response = await client.post(
                f"{CLIP_VECTORIZER_URL}/vectors_img",
                files=files
            )
            clip_response.raise_for_status()
            vector = clip_response.json()["result"]
        except httpx.HTTPError as e:
            raise HTTPException(status_code=503, detail=f"CLIP vectorizer error: {str(e)}")
    
    vectorize_time = (time.perf_counter() - vectorize_start) * 1000
    
    # Insert into VortexDB using the Python client
    def do_insert():
        db = get_vortexdb_client()
        try:
            point_id = db.insert(
                vector=DenseVector(vector),
                payload=Payload.image(file_path),
            )
            return point_id
        finally:
            db.close()
    
    insert_start = time.perf_counter()
    loop = asyncio.get_event_loop()
    try:
        point_id = await loop.run_in_executor(executor, do_insert)
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"VortexDB error: {str(e)}")
    
    insert_time = (time.perf_counter() - insert_start) * 1000
    
    return IndexResponse(
        point_id=point_id,
        vectorize_time_ms=round(vectorize_time, 2),
        insert_time_ms=round(insert_time, 2)
    )


@app.post("/index-batch")
async def index_batch():
    """
    Index all images in the images directory
    """
    results = []
    errors = []
    
    if not os.path.exists(IMAGES_DIR):
        return {"indexed": 0, "errors": ["Images directory not found"]}
    
    image_files = [f for f in os.listdir(IMAGES_DIR) 
                   if f.lower().endswith(('.png', '.jpg', '.jpeg', '.webp', '.gif'))]
    
    loop = asyncio.get_event_loop()
    
    async with httpx.AsyncClient(timeout=120.0) as client:
        for filename in image_files:
            file_path = os.path.join(IMAGES_DIR, filename)
            try:
                with open(file_path, "rb") as f:
                    content = f.read()
                
                # Vectorize
                files = {"file": (filename, content, "image/jpeg")}
                clip_response = await client.post(
                    f"{CLIP_VECTORIZER_URL}/vectors_img",
                    files=files
                )
                clip_response.raise_for_status()
                vector = clip_response.json()["result"]
                
                # Insert using the Python client
                def do_insert(vector, file_path):
                    db = get_vortexdb_client()
                    try:
                        point_id = db.insert(
                            vector=DenseVector(vector),
                            payload=Payload.image(file_path),
                        )
                        return point_id
                    finally:
                        db.close()
                
                point_id = await loop.run_in_executor(
                    executor, do_insert, vector, file_path
                )
                results.append({"filename": filename, "point_id": point_id})
                
            except Exception as e:
                errors.append({"filename": filename, "error": str(e)})
    
    return {
        "indexed": len(results),
        "results": results,
        "errors": errors
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=3001)
