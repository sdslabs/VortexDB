FROM python:3.11-slim

WORKDIR /app

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY demo/clip-image-search/backend/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy and install VortexDB Python client
COPY client/python /tmp/vortexdb-client
RUN pip install --no-cache-dir /tmp/vortexdb-client && rm -rf /tmp/vortexdb-client

COPY demo/clip-image-search/backend/main.py .

# Create images directory
RUN mkdir -p /app/images

EXPOSE 3001

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "3001"]
