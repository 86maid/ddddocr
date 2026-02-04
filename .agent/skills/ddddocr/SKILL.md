---
name: ddddocr
description: "DDDDOCR OCR recognition service with MCP protocol support. Provides optical character recognition, object detection, and slide matching capabilities. Use for: Recognizing text from captcha images, Detecting objects/text regions in images, Matching slide positions for verification codes, Performing any OCR-related tasks through MCP protocol."
---

# DDDDOCR Service

## Quick Start

Start the ddddocr service with all features enabled:

```bash
python scripts/start_ddddocr.py
```

The script automatically:
- Checks if service is already running
- Downloads the latest ddddocr binary for current platform if needed
- Starts service with ocr, det, slide, and mcp features
- Binds to 127.0.0.1:8000 by default

## Command Line Tools

Use the provided scripts for quick OCR operations:

### OCR Recognition

```bash
python scripts/ocr.py <image_path> [--color-filter FILTER] [--charset-range RANGE] [--text-only]
```

**Examples:**
```bash
python scripts/ocr.py image/3.png
python scripts/ocr.py image/3.png --text-only
python scripts/ocr.py image/3.png --color-filter green --charset-range "0123456789"
```

### Object Detection

```bash
python scripts/det.py <image_path> [--json]
```

**Examples:**
```bash
python scripts/det.py image/3.png
python scripts/det.py image/3.png --json
```

### Slide Matching

```bash
python scripts/slide.py <target_path> <background_path> [--algorithm match|comparison] [--simple-target] [--json]
```

**Examples:**
```bash
python scripts/slide.py image/su.png image/bg.png
python scripts/slide.py image/su.png image/bg.png --algorithm comparison
python scripts/slide.py image/target.png image/bg.png --simple-target --json
```

## Core Capabilities

### 1. OCR Recognition

Recognize text from images, supports color filtering and character range specification.

**Use cases:**
- Captcha recognition (numeric, alphanumeric, Chinese)
- Text extraction from images
- Custom character set recognition

**Endpoint:** `POST /ocr`

### 2. Object Detection

Detect text regions and objects in images.

**Use cases:**
- Point-and-click captcha verification
- Text region localization
- Multiple object detection

**Endpoint:** `POST /det`

### 3. Slide Matching

Match slide images with background positions.

**Algorithm 1 (slide-match):** Template matching for transparent slides
**Algorithm 2 (slide-comparison):** Difference-based comparison

**Use cases:**
- Slide captcha verification
- Image positioning

**Endpoints:** `POST /slide-match`, `POST /slide-comparison`

## MCP Protocol

The service implements the Model Context Protocol for AI agent integration.

**Endpoint:** `POST http://127.0.0.1:8000/mcp`

**Available MCP tools:**
- `ocr` - OCR recognition with optional color filtering and character range
- `det` - Object detection returning bounding boxes
- `slide_match` - Slide matching (algorithm 1)
- `slide_comparison` - Slide comparison (algorithm 2)

See [references/mcp.md](references/mcp.md) for MCP protocol details.

## REST API

The service also provides a REST API:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/ocr` | POST | OCR recognition |
| `/det` | POST | Object detection |
| `/slide-match` | POST | Slide matching |
| `/slide-comparison` | POST | Slide comparison |
| `/status` | GET | Service status |
| `/docs` | GET | Swagger UI documentation |

See [references/api.md](references/api.md) for detailed API documentation.

## Usage Examples

### OCR Recognition

```python
import requests
import base64

with open("image.png", "rb") as f:
    image_b64 = base64.b64encode(f.read()).decode()

response = requests.post("http://127.0.0.1:8000/ocr", json={
    "image": image_b64,
    "color_filter": "green",
    "charset_range": "0123456789"
})

print(response.json())
```

### Object Detection

```python
response = requests.post("http://127.0.0.1:8000/det", json={
    "image": image_b64
})

print(response.json())
```

### Slide Matching

```python
with open("target.png", "rb") as f:
    target_b64 = base64.b64encode(f.read()).decode()
with open("background.png", "rb") as f:
    bg_b64 = base64.b64encode(f.read()).decode()

response = requests.post("http://127.0.0.1:8000/slide-match", json={
    "target_image": target_b64,
    "background_image": bg_b64,
    "simple_target": True
})

print(response.json())
```

## Color Filtering

Supported presets: red, blue, green, yellow, orange, purple, cyan, black, white, gray

HSV ranges can also be specified as array of tuples: `[(min_h, min_s, min_v), (max_h, max_s, max_v)]`

## Character Range Values

| Value | Description |
|-------|-------------|
| 0 | Pure integers 0-9 |
| 1 | Pure lowercase a-z |
| 2 | Pure uppercase A-Z |
| 3 | Lowercase + Uppercase |
| 4 | Lowercase + 0-9 |
| 5 | Uppercase + 0-9 |
| 6 | Lowercase + Uppercase + 0-9 |
| 7 | Default full character set |

Custom string can also be used: `"0123456789+-x/=?"`

## Service Status

Check if service is running:

```bash
curl http://127.0.0.1:8000/status
```

Response:
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "service_status": "running",
    "enabled_features": ["ocr", "det", "slide", "mcp"]
  }
}
```
