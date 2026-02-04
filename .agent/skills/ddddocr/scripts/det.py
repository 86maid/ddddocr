#!/usr/bin/env python3
import sys
import base64
import argparse
from pathlib import Path


def detection(image_path, endpoint="http://127.0.0.1:8000/det"):
    import requests
    
    image_path = Path(image_path)
    
    if not image_path.exists():
        print(f"Error: Image file not found: {image_path}", file=sys.stderr)
        sys.exit(1)
    
    with open(image_path, "rb") as f:
        image_b64 = base64.b64encode(f.read()).decode()
    
    payload = {"image": image_b64}
    
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
    parser = argparse.ArgumentParser(description="Object detection using ddddocr service")
    parser.add_argument("image_path", help="Path to the image file")
    parser.add_argument("--endpoint", default="http://127.0.0.1:8000/det", help="Detection endpoint URL")
    parser.add_argument("--json", action="store_true", help="Output as raw JSON")
    
    args = parser.parse_args()
    
    result = detection(args.image_path, endpoint=args.endpoint)
    
    if args.json:
        import json
        print(json.dumps(result))
    else:
        bboxes = result.get("bboxes", [])
        print(f"Found {len(bboxes)} object(s):")
        for i, bbox in enumerate(bboxes, 1):
            print(f"  {i}. [{bbox[0]}, {bbox[1]}, {bbox[2]}, {bbox[3]}]")


if __name__ == "__main__":
    main()
