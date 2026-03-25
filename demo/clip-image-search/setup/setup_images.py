#!/usr/bin/env python3
"""
Setup script that downloads sample images and indexes them into VortexDB.
Runs as a Docker container after services are ready.
"""

import os
import sys
import time
import httpx
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

# Configuration from environment
CLIP_URL = os.getenv("CLIP_VECTORIZER_URL", "http://clip-vectorizer:8080")
VORTEXDB_URL = os.getenv("VORTEXDB_URL", "http://vortexdb:3000")
IMAGES_DIR = os.getenv("IMAGES_DIR", "/app/images")
IMAGE_COUNT = int(os.getenv("IMAGE_COUNT", "100"))

# Categories for diverse images
CATEGORIES = [
    "nature", "city", "people", "food", "tech", "animal", "architecture",
    "art", "travel", "sports", "fashion", "music", "business", "health",
    "ocean", "mountain", "forest", "desert", "snow", "beach", "sunset",
    "car", "bike", "train", "plane", "boat", "street", "building",
    "cat", "dog", "bird", "flower", "tree", "sky", "cloud",
    "coffee", "book", "laptop", "phone", "camera", "guitar", "piano"
]


def print_banner():
    print("=" * 60)
    print("  CLIP Image Search Demo - Auto Setup")
    print("=" * 60)
    print()


def wait_for_services(max_retries=60, delay=5):
    """Wait for CLIP and VortexDB services to be ready."""
    print("⏳ Waiting for services to be ready...")
    
    for attempt in range(max_retries):
        clip_ready = False
        vortex_ready = False
        
        try:
            with httpx.Client(timeout=5.0) as client:
                # Check CLIP
                try:
                    resp = client.get(f"{CLIP_URL}/docs")
                    clip_ready = resp.status_code == 200
                except Exception:
                    pass
                
                # Check VortexDB
                try:
                    resp = client.get(f"{VORTEXDB_URL}/health")
                    vortex_ready = resp.status_code == 200
                except Exception:
                    pass
        except Exception:
            pass
        
        if clip_ready and vortex_ready:
            print(f"✅ CLIP Vectorizer ready at {CLIP_URL}")
            print(f"✅ VortexDB ready at {VORTEXDB_URL}")
            return True
        
        status = []
        if clip_ready:
            status.append("CLIP ✓")
        else:
            status.append("CLIP ✗")
        if vortex_ready:
            status.append("VortexDB ✓")
        else:
            status.append("VortexDB ✗")
        
        print(f"   Attempt {attempt + 1}/{max_retries}: {' | '.join(status)}")
        time.sleep(delay)
    
    print("❌ Services did not become ready in time")
    return False


def download_image(seed: str, filepath: str) -> bool:
    """Download a single image from Lorem Picsum."""
    if os.path.exists(filepath):
        return True
    
    url = f"https://picsum.photos/seed/{seed}/640/640"
    try:
        with httpx.Client(timeout=30.0, follow_redirects=True) as client:
            resp = client.get(url)
            if resp.status_code == 200 and len(resp.content) > 1024:
                with open(filepath, "wb") as f:
                    f.write(resp.content)
                return True
    except Exception as e:
        print(f"   Download error for {seed}: {e}")
    return False


def download_images(count: int) -> int:
    """Download sample images from Lorem Picsum."""
    print()
    print("=" * 60)
    print(f"  Downloading {count} sample images...")
    print("=" * 60)
    print()
    
    os.makedirs(IMAGES_DIR, exist_ok=True)
    
    downloaded = 0
    i = 1
    
    while downloaded < count and i <= count * 2:
        cat_index = i % len(CATEGORIES)
        category = CATEGORIES[cat_index]
        seed = f"{category}{i}"
        filename = f"{category}_{i:03d}.jpg"
        filepath = os.path.join(IMAGES_DIR, filename)
        
        if download_image(seed, filepath):
            downloaded += 1
            if downloaded % 10 == 0:
                print(f"   Downloaded: {downloaded}/{count}")
        
        i += 1
        time.sleep(0.1)  # Be nice to the server
    
    print(f"\n✅ Downloaded {downloaded} images")
    return downloaded


def index_images() -> tuple[int, int]:
    """Index all images into VortexDB using CLIP."""
    print()
    print("=" * 60)
    print("  Indexing images into VortexDB...")
    print("=" * 60)
    print()
    
    image_files = list(Path(IMAGES_DIR).glob("*.jpg")) + \
                  list(Path(IMAGES_DIR).glob("*.jpeg")) + \
                  list(Path(IMAGES_DIR).glob("*.png"))
    
    if not image_files:
        print("❌ No images found to index")
        return 0, 0
    
    print(f"   Found {len(image_files)} images to index")
    
    indexed = 0
    failed = 0
    total_time = 0
    
    with httpx.Client(timeout=120.0) as client:
        for i, img_path in enumerate(image_files, 1):
            try:
                start = time.perf_counter()
                
                # Vectorize with CLIP
                with open(img_path, "rb") as f:
                    files = {"file": (img_path.name, f, "image/jpeg")}
                    clip_resp = client.post(f"{CLIP_URL}/vectors_img", files=files)
                
                if clip_resp.status_code != 200:
                    print(f"   [{i}/{len(image_files)}] ❌ {img_path.name} (CLIP error)")
                    failed += 1
                    continue
                
                vector = clip_resp.json().get("result")
                if not vector:
                    print(f"   [{i}/{len(image_files)}] ❌ {img_path.name} (no vector)")
                    failed += 1
                    continue
                
                # Insert into VortexDB
                insert_resp = client.post(
                    f"{VORTEXDB_URL}/points",
                    json={
                        "vector": vector,
                        "payload": {
                            "content_type": "Image",
                            "content": str(img_path)
                        }
                    }
                )
                
                elapsed = (time.perf_counter() - start) * 1000
                total_time += elapsed
                
                if insert_resp.status_code in (200, 201):
                    indexed += 1
                    if indexed % 10 == 0:
                        avg = total_time / indexed
                        print(f"   [{i}/{len(image_files)}] Indexed {indexed} images (avg: {avg:.0f}ms)")
                else:
                    print(f"   [{i}/{len(image_files)}] ❌ {img_path.name} (DB error: {insert_resp.status_code})")
                    failed += 1
                    
            except Exception as e:
                print(f"   [{i}/{len(image_files)}] ❌ {img_path.name}: {e}")
                failed += 1
    
    print()
    print(f"✅ Indexed: {indexed} images")
    if failed > 0:
        print(f"❌ Failed: {failed} images")
    if indexed > 0:
        print(f"⚡ Average time per image: {total_time / indexed:.0f}ms")
    
    return indexed, failed


def main():
    print_banner()
    
    # Wait for services
    if not wait_for_services():
        sys.exit(1)
    
    print()
    
    # Download images
    downloaded = download_images(IMAGE_COUNT)
    
    if downloaded == 0:
        print("❌ No images downloaded, exiting")
        sys.exit(1)
    
    # Index images
    indexed, failed = index_images()
    
    print()
    print("=" * 60)
    print("  🎉 Setup Complete!")
    print("=" * 60)
    print()
    print("  Open http://localhost:3000 to try the demo")
    print()
    
    # Keep container running briefly so logs can be seen
    time.sleep(5)


if __name__ == "__main__":
    main()
