from typing import List

from openai import OpenAI

from src.config import Config


class Embedder:
    def __init__(self):
        self.client = OpenAI(
            api_key=Config.OPENAI_API_KEY,
            base_url=Config.OPENAI_BASE_URL,
        )
        self.model = Config.EMBEDDING_MODEL

    def embed(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for a list of texts."""
        if not texts:
            return []

        embeddings = []
        for i in range(0, len(texts), Config.EMBEDDING_BATCH_SIZE):
            batch = texts[i : i + Config.EMBEDDING_BATCH_SIZE]
            response = self.client.embeddings.create(model=self.model, input=batch)
            embeddings.extend(item.embedding for item in response.data)

        return embeddings

    def embed_single(self, text: str) -> List[float]:
        """Generate embedding for a single text."""
        embeddings = self.embed([text])
        return embeddings[0] if embeddings else []
