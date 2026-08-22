FROM python:3.11-slim

WORKDIR /app

# Install curl for downloads and healthchecks
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY setup_images.py .

# Default to 100 images
ENV IMAGE_COUNT=100
ENV CLIP_VECTORIZER_URL=http://clip-vectorizer:8080
ENV VORTEXDB_URL=http://vortexdb:3000
ENV IMAGES_DIR=/app/images

CMD ["python", "setup_images.py"]
