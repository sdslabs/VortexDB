import os
import sys
from pathlib import Path

from src.chunker import chunk_text
from src.config import Config
from src.embedder import Embedder
from src.extractor import extract_text
from src.vectorstore import VectorStore

SUPPORTED_EXTENSIONS = {".txt", ".md", ".pdf", ".docx", ".csv"}


def keep_supported_files(directory: str) -> list[Path]:
    file_paths = []
    for root, _, files in os.walk(directory):
        for file in files:
            file_path = Path(root) / file
            if file_path.suffix.lower() in SUPPORTED_EXTENSIONS:
                file_paths.append(file_path)
    return file_paths


def parse_files(directory: str) -> list[dict]:
    parsed_files = []
    for file_path in keep_supported_files(directory):
        extracted_text = extract_text(file_path)
        chunks = chunk_text(
            extracted_text,
            chunk_size=Config.CHUNK_SIZE,
            chunk_overlap=Config.CHUNK_OVERLAP,
        )
        parsed_files.append({"file_path": str(file_path), "chunks": chunks})
    return parsed_files


def embed_chunks(parsed_files: list[dict]) -> list[dict]:
    embedder = Embedder()
    embedded_files = []
    for parsed_file in parsed_files:
        embedded_files.append(
            {
                "file_path": parsed_file["file_path"],
                "chunks": parsed_file["chunks"],
                "embeddings": embedder.embed(parsed_file["chunks"]),
            }
        )
    return embedded_files


def ingest_documents() -> None:
    directory = os.path.expanduser(Config.DOCS_DIRECTORY)

    if not Config.OPENAI_API_KEY:
        print("Error: OPENAI_API_KEY not set.")
        sys.exit(1)

    if not os.path.isdir(directory):
        print(f"Error: Documents directory not found: {directory}")
        sys.exit(1)

    parsed_files = parse_files(directory)
    if not parsed_files:
        print(f"No supported documents found in {directory}. Skipping ingest.")
        return

    embedded_files = embed_chunks(parsed_files)
    vector_store = VectorStore(Config.VORTEXDB_HOST, Config.VORTEXDB_PORT)

    total_inserted = 0
    for embedded_file in embedded_files:
        inserted = vector_store.insert_batch(
            embedded_file["embeddings"],
            embedded_file["chunks"],
            embedded_file["file_path"],
        )
        total_inserted += inserted
        print(f"Ingested {inserted} chunks from {embedded_file['file_path']}")

    print(
        f"Done. Ingested {total_inserted} chunks from {len(embedded_files)} files "
        f"into VortexDB at {Config.VORTEXDB_HOST}:{Config.VORTEXDB_PORT}"
    )


if __name__ == "__main__":
    ingest_documents()
