import httpx
from typing import List, Dict
from src.config import Config


class VectorStore:
    def __init__(self, host: str, port: int):
        self.base_url = f"http://{host}:{port}"
        self.vector_size = Config.VECTOR_SIZE
    
    def _get_client(self) -> httpx.Client:
        return httpx.Client(base_url=self.base_url, timeout=60.0)
    
    def insert_batch(self, vectors: List[List[float]], texts: List[str], filename: str) -> int:
        """Batch insert vectors using VortexDB's batch insert endpoint."""
        client = self._get_client()
        
        points = []
        for i, (vector, text) in enumerate(zip(vectors, texts)):
            points.append({
                "vector": vector,
                "payload": {
                    "content_type": "Text",
                    "content": text
                }
            })
        
        response = client.post("/points/batch", json={"points": points})
        
        if response.status_code != 200:
            raise Exception(f"Batch insert failed: {response.text}")
        
        data = response.json()
        return data.get("inserted", len(points))
    
    def search(self, query_vector: List[float], top_k: int = 5) -> List[Dict]:
        """Search for similar vectors."""
        client = self._get_client()
        
        response = client.post("/points/search", json={
            "vector": query_vector,
            "similarity": "Cosine",
            "limit": top_k
        })
        
        if response.status_code != 200:
            return []
        
        data = response.json()
        results = []
        
        for point_id in data.get("results", []):
            point_response = client.get(f"/points/{point_id}")
            if point_response.status_code == 200:
                point = point_response.json()
                payload = point.get("payload", {})
                results.append({
                    "id": point_id,
                    "text": payload.get("content", ""),
                    "filename": "",
                    "score": 1.0
                })
        
        return results
    
    def get_point_count(self) -> int:
        return 0
    
    def clear(self):
        pass
    
    def get_info(self) -> Dict:
        return {"points_count": 0, "status": "ok"}
