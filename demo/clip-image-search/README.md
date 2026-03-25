# CLIP Image Search Demo

A simple image search demo powered by **VortexDB** using CLIP embedding model.
Search images using natural language descriptions and see results update as you type.


## Quick Start


```bash
# From the demo directory
cd demo/clip-image-search

# Also downloads 100 demo images
sudo docker compose up
```
The setup container will:
1. Wait for CLIP and VortexDB to be ready
2. Download 100 sample images from Lorem Picsum
3. Vectorize and index them into VortexDB
4. Exit when complete

To change the number of images, edit `docker-compose.yml` and set `IMAGE_COUNT=50` (or any number).

### Manual Setup (Optional)

If you prefer to run setup separately:

```bash
# Start services only
sudo docker compose up -d clip-vectorizer vortexdb backend frontend

# Run setup manually with custom count
./setup.sh --count 50
```

## Adding Your Own Images

### Via the Backend API

```bash
# Index a single image
curl -X POST http://localhost:3001/index \
  -F "file=@/path/to/your/image.jpg"
```

### Via the Setup Script

1. Copy images to `demo/clip-image-search/backend/images/`
2. Run `./setup.sh` (will index existing images without downloading new ones)
