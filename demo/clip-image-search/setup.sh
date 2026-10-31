#!/bin/bash
set -e

# CLIP Image Search Demo - Setup Script
# Downloads sample images and indexes them into VortexDB

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGES_DIR="${SCRIPT_DIR}/backend/images"
CLIP_URL="${CLIP_VECTORIZER_URL:-http://localhost:5000}"
VORTEXDB_URL="${VORTEXDB_URL:-http://localhost:8081}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     CLIP Image Search Demo - Setup Script                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Create images directory
mkdir -p "$IMAGES_DIR"

# Function to check if services are running
check_services() {
    echo -e "${YELLOW}Checking services...${NC}"
    
    # Check CLIP Vectorizer
    if curl -s "${CLIP_URL}/docs" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ CLIP Vectorizer is running at ${CLIP_URL}${NC}"
    else
        echo -e "${RED}✗ CLIP Vectorizer is not reachable at ${CLIP_URL}${NC}"
        echo -e "${YELLOW}  Make sure docker compose is running: sudo docker compose up -d${NC}"
        return 1
    fi
    
    # Check VortexDB
    if curl -s "${VORTEXDB_URL}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ VortexDB is running at ${VORTEXDB_URL}${NC}"
    else
        echo -e "${RED}✗ VortexDB is not reachable at ${VORTEXDB_URL}${NC}"
        echo -e "${YELLOW}  Make sure docker compose is running: sudo docker compose up -d${NC}"
        return 1
    fi
    
    return 0
}

# Function to download images from Lorem Picsum (free, no API key needed)
download_images() {
    local count=${1:-100}
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Downloading ${count} sample images from Lorem Picsum...${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    # Categories for diverse images (using seed words for consistent results)
    local categories=(
        "nature" "city" "people" "food" "tech" "animal" "architecture" 
        "art" "travel" "sports" "fashion" "music" "business" "health"
        "ocean" "mountain" "forest" "desert" "snow" "beach" "sunset"
        "car" "bike" "train" "plane" "boat" "street" "building"
        "cat" "dog" "bird" "flower" "tree" "sky" "cloud"
        "coffee" "book" "laptop" "phone" "camera" "guitar" "piano"
    )
    
    local downloaded=0
    local failed=0
    local i=1
    
    while [ $downloaded -lt $count ]; do
        # Get category based on current index
        local cat_index=$((i % ${#categories[@]}))
        local category="${categories[$cat_index]}"
        local seed="${category}${i}"
        local filename="${category}_$(printf '%03d' $i).jpg"
        local filepath="${IMAGES_DIR}/${filename}"
        
        if [ -f "$filepath" ]; then
            echo -e "  [${downloaded}/${count}] ${YELLOW}Skipping${NC} ${filename} (already exists)"
            ((downloaded++))
        else
            # Download from Lorem Picsum with seed for reproducible images
            # Using 640x640 for good quality but reasonable size
            local url="https://picsum.photos/seed/${seed}/640/640"
            
            if curl -sL -o "$filepath" "$url" 2>/dev/null; then
                # Verify it's a valid image (check file size > 1KB)
                if [ -s "$filepath" ] && [ $(stat -f%z "$filepath" 2>/dev/null || stat -c%s "$filepath" 2>/dev/null) -gt 1024 ]; then
                    echo -e "  [${downloaded}/${count}] ${GREEN}Downloaded${NC} ${filename}"
                    ((downloaded++))
                else
                    rm -f "$filepath"
                    echo -e "  [${downloaded}/${count}] ${RED}Failed${NC} ${filename} (invalid image)"
                    ((failed++))
                fi
            else
                echo -e "  [${downloaded}/${count}] ${RED}Failed${NC} ${filename}"
                ((failed++))
            fi
        fi
        
        ((i++))
        
        # Prevent infinite loop
        if [ $i -gt $((count * 2)) ]; then
            echo -e "${YELLOW}Warning: Too many failures, stopping download${NC}"
            break
        fi
        
        # Small delay to be nice to the server
        sleep 0.1
    done
    
    echo ""
    echo -e "${GREEN}Downloaded: ${downloaded} images${NC}"
    if [ $failed -gt 0 ]; then
        echo -e "${RED}Failed: ${failed} images${NC}"
    fi
}

# Function to index images into VortexDB
index_images() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Indexing images into VortexDB...${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local total=0
    local indexed=0
    local failed=0
    local total_vectorize_time=0
    local total_insert_time=0
    
    # Count total images
    for img in "${IMAGES_DIR}"/*.{jpg,jpeg,png,webp,gif} 2>/dev/null; do
        [ -f "$img" ] && ((total++))
    done
    
    if [ $total -eq 0 ]; then
        echo -e "${RED}No images found in ${IMAGES_DIR}${NC}"
        return 1
    fi
    
    echo -e "Found ${total} images to index"
    echo ""
    
    for img in "${IMAGES_DIR}"/*.{jpg,jpeg,png,webp,gif} 2>/dev/null; do
        [ -f "$img" ] || continue
        
        local filename=$(basename "$img")
        local current=$((indexed + failed + 1))
        
        # Step 1: Vectorize with CLIP
        local vec_start=$(date +%s%3N)
        local vector_response=$(curl -s -X POST "${CLIP_URL}/vectors_img" \
            -F "file=@${img}" \
            -H "accept: application/json")
        local vec_end=$(date +%s%3N)
        local vec_time=$((vec_end - vec_start))
        
        # Extract vector from response
        local vector=$(echo "$vector_response" | grep -o '"result":\[[^]]*\]' | sed 's/"result"://')
        
        if [ -z "$vector" ] || [ "$vector" = "null" ]; then
            echo -e "  [${current}/${total}] ${RED}✗${NC} ${filename} (vectorization failed)"
            ((failed++))
            continue
        fi
        
        # Step 2: Insert into VortexDB
        local ins_start=$(date +%s%3N)
        local insert_response=$(curl -s -X POST "${VORTEXDB_URL}/points" \
            -H "Content-Type: application/json" \
            -d "{
                \"vector\": ${vector},
                \"payload\": {
                    \"content_type\": \"Image\",
                    \"content\": \"${img}\"
                }
            }")
        local ins_end=$(date +%s%3N)
        local ins_time=$((ins_end - ins_start))
        
        # Check if insert was successful
        local point_id=$(echo "$insert_response" | grep -o '"point_id":"[^"]*"' | sed 's/"point_id":"\([^"]*\)"/\1/')
        
        if [ -n "$point_id" ]; then
            echo -e "  [${current}/${total}] ${GREEN}✓${NC} ${filename} (CLIP: ${vec_time}ms | DB: ${ins_time}ms)"
            ((indexed++))
            total_vectorize_time=$((total_vectorize_time + vec_time))
            total_insert_time=$((total_insert_time + ins_time))
        else
            echo -e "  [${current}/${total}] ${RED}✗${NC} ${filename} (insert failed)"
            ((failed++))
        fi
    done
    
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Indexing Complete!${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ${GREEN}Indexed:${NC} ${indexed} images"
    if [ $failed -gt 0 ]; then
        echo -e "  ${RED}Failed:${NC}  ${failed} images"
    fi
    
    if [ $indexed -gt 0 ]; then
        local avg_vec=$((total_vectorize_time / indexed))
        local avg_ins=$((total_insert_time / indexed))
        echo ""
        echo -e "  ${BLUE}Performance:${NC}"
        echo -e "    Avg CLIP vectorization: ${avg_vec}ms"
        echo -e "    Avg VortexDB insert:    ${avg_ins}ms"
        echo -e "    Total time:             $((total_vectorize_time + total_insert_time))ms"
    fi
}

# Main execution
main() {
    local skip_download=false
    local image_count=100
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-download)
                skip_download=true
                shift
                ;;
            --count)
                image_count=$2
                shift 2
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-download    Skip image download, only index existing images"
                echo "  --count N          Number of images to download (default: 100)"
                echo "  -h, --help         Show this help message"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Check services first
    if ! check_services; then
        echo ""
        echo -e "${RED}Please start the services first:${NC}"
        echo -e "  cd ${SCRIPT_DIR}"
        echo -e "  sudo docker compose up -d"
        echo -e "  # Wait ~60 seconds for CLIP model to load"
        echo -e "  $0"
        exit 1
    fi
    
    # Download images
    if [ "$skip_download" = false ]; then
        download_images $image_count
    fi
    
    # Index images
    index_images
    
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Setup Complete! Open http://localhost:3000 to try it out  ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

main "$@"
