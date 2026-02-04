# DDDDOCR API Reference

This document describes the REST API endpoints available in the ddddocr server.

## Base URL

```
http://127.0.0.1:8000
```

## Endpoints

### 1. `/status` - Service Status

Get the current status of the ddddocr service.

**Method:** `GET`

**Response:**
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "service_status": "running",
    "enabled_features": ["ocr", "det", "slide"]
  }
}
```

### 2. `/ocr` - OCR Recognition

Perform optical character recognition on an image.

**Method:** `POST`

**Request:**
```json
{
  "image": "base64_encoded_image",
  "color_filter": "green",
  "charset_range": "0123456789+-x/="
}
```

**Parameters:**
- `image` (required): Base64 encoded image data
- `color_filter` (optional): Color filter to apply. Presets: red, blue, green, yellow, orange, purple, cyan, black, white, gray. Can also use HSV ranges.
- `charset_range` (optional): Character range for recognition. Values 0-7 for built-in ranges, or custom string (e.g., "0123456789+-x/=")
- `simple_target` (optional): For simple slide matching

**Response:**
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "text": "九乘六等于？",
    "probability": null
  }
}
```

### 3. `/det` - Object Detection

Detect objects in an image.

**Method:** `POST`

**Request:**
```json
{
  "image": "base64_encoded_image"
}
```

**Response:**
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "bboxes": [[80, 3, 98, 21], [56, 6, 76, 25]]
  }
}
```

### 4. `/slide-match` - Slide Matching (Algorithm 1)

Match a small slide image with its position in the background image.

**Method:** `POST`

**Request:**
```json
{
  "target_image": "base64_encoded_target",
  "background_image": "base64_encoded_background",
  "simple_target": true
}
```

**Parameters:**
- `target_image` (required): Base64 encoded small slide image
- `background_image` (required): Base64 encoded background with hole
- `simple_target` (optional): Use for images without transparent background

**Response:**
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "target": [215, 45, 261, 91],
    "target_x": 0,
    "target_y": 45
  }
}
```

### 5. `/slide-comparison` - Slide Comparison (Algorithm 2)

Compare two images to find the slide position.

**Method:** `POST`

**Request:**
```json
{
  "target_image": "base64_encoded_target",
  "background_image": "base64_encoded_background"
}
```

**Response:**
```json
{
  "code": 200,
  "msg": "success",
  "data": {
    "x": 144,
    "y": 76
  }
}
```

### 6. `/docs` - API Documentation

Swagger UI documentation.

**Method:** `GET`

### 7. `/mcp` - MCP Protocol Endpoint

Model Context Protocol endpoint for AI agents.

**Method:** `POST`

See `mcp.md` for details.

## Character Range Values (for OCR)

| Value | Description                          |
|-------|--------------------------------------|
| 0     | Pure integers 0-9                    |
| 1     | Pure lowercase letters a-z           |
| 2     | Pure uppercase letters A-Z           |
| 3     | Lowercase a-z + Uppercase A-Z         |
| 4     | Lowercase a-z + Integers 0-9          |
| 5     | Uppercase A-Z + Integers 0-9          |
| 6     | Lowercase a-z + Uppercase A-Z + 0-9   |
| 7     | Default character set (excluding spaces) |

## Color Filter Presets

- `red`, `blue`, `green`, `yellow`, `orange`, `purple`, `cyan`, `black`, `white`, `gray`

HSV ranges can also be specified as array of tuples: `[(min_h, min_s, min_v), (max_h, max_s, max_v)]`
