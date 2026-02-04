#!/usr/bin/env python3
import sys
import base64
import argparse
from pathlib import Path


def slide_match(target_path, background_path, algorithm="match", endpoint="http://127.0.0.1:8000/slide-match", simple_target=False):
    import requests
    
    target_path = Path(target_path)
    background_path = Path(background_path)
    
    if not target_path.exists():
        print(f"Error: Target image not found: {target_path}", file=sys.stderr)
        sys.exit(1)
    
    if not background_path.exists():
        print(f"Error: Background image not found: {background_path}", file=sys.stderr)
        sys.exit(1)
    
    with open(target_path, "rb") as f:
        target_b64 = base64.b64encode(f.read()).decode()
    
    with open(background_path, "rb") as f:
        bg_b64 = base64.b64encode(f.read()).decode()
    
    payload = {
        "target_image": target_b64,
        "background_image": bg_b64
    }
    
    if simple_target:
        payload["simple_target"] = True
    
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


def slide_comparison(target_path, background_path, endpoint="http://127.0.0.1:8000/slide-comparison"):
    import requests
    
    target_path = Path(target_path)
    background_path = Path(background_path)
    
    if not target_path.exists():
        print(f"Error: Target image not found: {target_path}", file=sys.stderr)
        sys.exit(1)
    
    if not background_path.exists():
        print(f"Error: Background image not found: {background_path}", file=sys.stderr)
        sys.exit(1)
    
    with open(target_path, "rb") as f:
        target_b64 = base64.b64encode(f.read()).decode()
    
    with open(background_path, "rb") as f:
        bg_b64 = base64.b64encode(f.read()).decode()
    
    payload = {
        "target_image": target_b64,
        "background_image": bg_b64
    }
    
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
    parser = argparse.ArgumentParser(description="Slide matching using ddddocr service")
    parser.add_argument("target_path", help="Path to the target/slide image")
    parser.add_argument("background_path", help="Path to the background image")
    parser.add_argument("--algorithm", choices=["match", "comparison"], default="match", help="Algorithm to use (match or comparison)")
    parser.add_argument("--endpoint", help="Custom endpoint URL (default: /slide-match or /slide-comparison)")
    parser.add_argument("--simple-target", action="store_true", help="Use simple matching for non-transparent target images")
    parser.add_argument("--json", action="store_true", help="Output as raw JSON")
    
    args = parser.parse_args()
    
    if args.algorithm == "match":
        endpoint = args.endpoint or "http://127.0.0.1:8000/slide-match"
        result = slide_match(args.target_path, args.background_path, endpoint=endpoint, simple_target=args.simple_target)
    else:
        endpoint = args.endpoint or "http://127.0.0.1:8000/slide-comparison"
        result = slide_comparison(args.target_path, args.background_path, endpoint=endpoint)
    
    if args.json:
        import json
        print(json.dumps(result))
    else:
        if "target" in result:
            bbox = result["target"]
            x = result.get("target_x", bbox[0])
            y = result.get("target_y", bbox[1])
            print(f"Position: x={x}, y={y}")
            print(f"Bounding box: [{bbox[0]}, {bbox[1]}, {bbox[2]}, {bbox[3]}]")
        elif "x" in result:
            print(f"Position: x={result['x']}, y={result['y']}")


if __name__ == "__main__":
    main()
