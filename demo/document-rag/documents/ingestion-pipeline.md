# Ingestion Pipeline

The document ingestion pipeline scans a directory for supported files: TXT, MD, PDF, DOCX, and CSV.
Each file is extracted to plain text, split into overlapping chunks, and embedded using a Cloudflare Workers AI embedding model.
Vectors are stored in VortexDB with a text payload for retrieval during chat.
