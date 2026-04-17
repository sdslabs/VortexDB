import re
from typing import List


def chunk_text(text: str, chunk_size: int = 512, chunk_overlap: int = 50) -> List[str]:
    """
    Split text into overlapping chunks.
    """
    if not text or not text.strip():
        return []
    
    text = re.sub(r'\s+', ' ', text).strip()
    
    chunks = []
    start = 0
    text_len = len(text)
    
    while start < text_len:
        end = start + chunk_size
        chunk = text[start:end]
        
        if end < text_len:
            last_period = chunk.rfind('. ')
            last_newline = chunk.rfind('\n')
            split_pos = max(last_period, last_newline)
            
            if split_pos > chunk_size // 2:
                chunk = chunk[:split_pos + 1]
                end = start + split_pos + 1
        
        chunks.append(chunk.strip())
        start = end - chunk_overlap if end < text_len else text_len
    
    return [c for c in chunks if c]
