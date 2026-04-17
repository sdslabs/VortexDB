import openai
from openai import OpenAI
from typing import List
from src.config import Config


class Embedder:
    def __init__(self, api_key: str):
        self.client = OpenAI(api_key=api_key)
        self.model = Config.EMBEDDING_MODEL
    
    def embed(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for a list of texts."""
        if not texts:
            return []
        
        response = self.client.embeddings.create(
            model=self.model,
            input=texts
        )
        
        return [item.embedding for item in response.data]
    
    def embed_single(self, text: str) -> List[float]:
        """Generate embedding for a single text."""
        embeddings = self.embed([text])
        return embeddings[0] if embeddings else []
