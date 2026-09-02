import os
from dotenv import load_dotenv

load_dotenv()

_account_id = os.getenv("ACCOUNT_ID", "")


class Config:
    OPENAI_API_KEY: str = os.getenv("OPENAI_API_KEY", "")
    OPENAI_BASE_URL: str = (
        f"https://api.cloudflare.com/client/v4/accounts/{_account_id}/ai/v1"
    )
    VORTEXDB_HOST: str = os.getenv("VORTEXDB_HOST", "localhost")
    VORTEXDB_PORT: int = int(os.getenv("VORTEXDB_PORT", "3034"))
    EMBEDDING_MODEL: str = os.getenv("EMBEDDING_MODEL", "")
    LLM_MODEL: str = os.getenv("LLM_MODEL", "")
    CHUNK_SIZE: int = int(os.getenv("CHUNK_SIZE", "512"))
    CHUNK_OVERLAP: int = int(os.getenv("CHUNK_OVERLAP", "50"))
    TOP_K: int = int(os.getenv("TOP_K", "5"))
    VECTOR_SIZE: int = int(os.getenv("VECTOR_SIZE", ""))
    DOCS_DIRECTORY: str = os.getenv("DOCS_DIRECTORY", "./documents")
    EMBEDDING_BATCH_SIZE: int = int(os.getenv("EMBEDDING_BATCH_SIZE", "64"))
