#!/usr/bin/env python3
import sys
import base64
import argparse
from pathlib import Path


def ocr_recognition(image_path, endpoint="http://127.0.0.1:8000/ocr", color_filter=None, charset_range=None):
    import requests
    
    image_path = Path(image_path)
    
    if not image_path.exists():
        print(f"Error: Image file not found: {image_path}", file=sys.stderr)
        sys.exit(1)
    
    with open(image_path, "rb") as f:
        image_b64 = base64.b64encode(f.read()).decode()
    
    payload = {"image": image_b64}
    
    if color_filter:
        payload["color_filter"] = color_filter
    
    if charset_range:
        payload["charset_range"] = charset_range
    
    try:
        response = requests.post(endpoint, json=payload, timeout=30)
        response.raise_for_status()
        result = response.json()
        
        if result.get("code") == 200:
            data = result.get("data", {})
            return data
        else:
            print(f"Error: {result.get('msg', 'Unknown error')}", file=sys.stderr)
            sys.exit(1)
    except requests.exceptions.RequestException as e:
        print(f"Error: Failed to connect to ddddocr service: {e}", file=sys.stderr)
        print(f"Make sure the service is running: python scripts/start_ddddocr.py", file=sys.stderr)
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description="OCR recognition using ddddocr service")
    parser.add_argument("image_path", help="Path to the image file")
    parser.add_argument("--color-filter", help="Color filter to apply (red, blue, green, yellow, orange, purple, cyan, black, white, gray)")
    parser.add_argument("--charset-range", help="Character range for recognition (0-7 or custom string)")
    parser.add_argument("--endpoint", default="http://127.0.0.1:8000/ocr", help="OCR endpoint URL")
    parser.add_argument("--text-only", action="store_true", help="Output only the recognized text")
    
    args = parser.parse_args()
    
    result = ocr_recognition(
        args.image_path,
        endpoint=args.endpoint,
        color_filter=args.color_filter,
        charset_range=args.charset_range
    )
    
    if args.text_only:
        print(result.get("text", ""))
    else:
        print(f"Text: {result.get('text', '')}")
        if result.get("probability"):
            print(f"Probability: {result['probability']}")


if __name__ == "__main__":
    main()
