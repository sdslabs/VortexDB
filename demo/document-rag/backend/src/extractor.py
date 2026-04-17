from pathlib import Path
from PyPDF2 import PdfReader
import docx


SUPPORTED_EXTENSIONS = {'.txt', '.md', '.pdf', '.docx', '.csv'}


def extract_text(file_path: str) -> str:
    path = Path(file_path)
    ext = path.suffix.lower()
    
    if ext not in SUPPORTED_EXTENSIONS:
        raise ValueError(f"Unsupported format: {ext}")
    
    extractors = {
        '.txt': extract_txt,
        '.md': extract_markdown,
        '.pdf': extract_pdf,
        '.docx': extract_docx,
        '.csv': extract_csv,
    }
    
    return extractors[ext](file_path)


def extract_txt(file_path: str) -> str:
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        return f.read()


def extract_markdown(file_path: str) -> str:
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        return f.read()


def extract_pdf(file_path: str) -> str:
    reader = PdfReader(file_path)
    text_parts = []
    for page in reader.pages:
        text = page.extract_text()
        if text:
            text_parts.append(text)
    return "\n\n".join(text_parts)


def extract_docx(file_path: str) -> str:
    doc = docx.Document(file_path)
    paragraphs = [p.text for p in doc.paragraphs if p.text.strip()]
    return "\n\n".join(paragraphs)


def extract_csv(file_path: str) -> str:
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        return f.read()
