# DDDDOCR MCP Protocol

This document describes the Model Context Protocol (MCP) implementation in the ddddocr server.

## Endpoint

```
POST /mcp
```

## Supported Methods

### 1. `initialize`

Initialize the MCP connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "client-name",
      "version": "1.0.0"
    }
  }
}
```

### 2. `tools/list`

List available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "ocr",
        "description": "Perform OCR recognition on an image",
        "inputSchema": {
          "type": "object",
          "properties": {
            "image": {
              "type": "string",
              "description": "Base64 encoded image data"
            },
            "color_filter": {
              "type": "string",
              "description": "Color filter to apply"
            },
            "charset_range": {
              "type": "string",
              "description": "Character range for recognition"
            }
          },
          "required": ["image"]
        }
      },
      {
        "name": "det",
        "description": "Detect objects in an image",
        "inputSchema": {
          "type": "object",
          "properties": {
            "image": {
              "type": "string",
              "description": "Base64 encoded image data"
            }
          },
          "required": ["image"]
        }
      },
      {
        "name": "slide_match",
        "description": "Match slide with background",
        "inputSchema": {
          "type": "object",
          "properties": {
            "target_image": {
              "type": "string",
              "description": "Base64 encoded target image"
            },
            "background_image": {
              "type": "string",
              "description": "Base64 encoded background image"
            },
            "simple_target": {
              "type": "boolean",
              "description": "Use simple matching for non-transparent images"
            }
          },
          "required": ["target_image", "background_image"]
        }
      },
      {
        "name": "slide_comparison",
        "description": "Compare images to find slide position",
        "inputSchema": {
          "type": "object",
          "properties": {
            "target_image": {
              "type": "string",
              "description": "Base64 encoded target image"
            },
            "background_image": {
              "type": "string",
              "description": "Base64 encoded background image"
            }
          },
          "required": ["target_image", "background_image"]
        }
      }
    ]
  }
}
```

### 3. `tools/call`

Call a tool.

**Request Example (OCR):**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "ocr",
    "arguments": {
      "image": "base64_encoded_image",
      "color_filter": "green"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"probability\":null,\"text\":\"等于？\"}"
      }
    ],
    "isError": false
  }
}
```

**Request Example (Detection):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "det",
    "arguments": {
      "image": "base64_encoded_image"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"bboxes\":[[80,3,98,21],[56,6,76,25]]}"
      }
    ],
    "isError": false
  }
}
```

## Configuration

To enable MCP support, start the ddddocr server with the `--mcp` flag:

```bash
ddddocr --address 0.0.0.0:8000 --ocr --det --slide --mcp
```

To run ONLY MCP (no REST API), use `--only-mcp`:

```bash
ddddocr --address 0.0.0.0:8000 --ocr --det --slide --only-mcp
```

## Tool Descriptions

### `ocr`
Perform OCR recognition on an image. Supports color filtering and character range specification.

### `det`
Detect objects (text regions) in an image. Returns bounding box coordinates.

### `slide_match`
Match a small slide image with its position in the background image using template matching.

### `slide_comparison`
Compare two images to find the slide position using difference-based algorithm.
