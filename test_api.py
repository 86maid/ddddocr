import json
import base64
import requests

def test_routes(
    base_url="http://127.0.0.1:8000",
    image_path="./image/4.png",
    slide_match_target_path="./image/a.png",
    slide_match_background_path="./image/b.png",
    slide_comparison_target_path="./image/c.jpg",
    slide_comparison_background_path="./image/d.jpg",
):
    image_b64 = base64.b64encode(open(image_path, "rb").read()).decode()
    slide_target_b64 = base64.b64encode(
        open(slide_match_target_path, "rb").read()
    ).decode()
    slide_background_b64 = base64.b64encode(
        open(slide_match_background_path, "rb").read()
    ).decode()
    cmp_target_b64 = base64.b64encode(
        open(slide_comparison_target_path, "rb").read()
    ).decode()
    cmp_background_b64 = base64.b64encode(
        open(slide_comparison_background_path, "rb").read()
    ).decode()

    routes = [
        {"method": "get", "path": "/status", "json": None},
        {"method": "post", "path": "/ocr", "json": {"image": image_b64}},
        {
            "method": "post",
            "path": "/ocr",
            "json": {"image": image_b64, "color_filter": "green"},
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {"image": image_b64, "color_filter": ["green"]},
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {"image": image_b64, "color_filter": ["green", "blue"]},
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {
                "image": image_b64,
                "color_filter": [[[40, 50, 50], [80, 255, 255]]],
            },
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {
                "image": image_b64,
                "charset_range": "A九A乘A六A等A于A？A",
            },
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {
                "image": image_b64,
                "charset_range": "A九A乘A六A等A于A？A",
                "color_filter": "green",
            },
        },
        {
            "method": "post",
            "path": "/ocr",
            "json": {
                "image": image_b64,
                "charset_range": "A九A乘A六A等A于A？A",
                "probability": True,
            },
        },
        {"method": "post", "path": "/det", "json": {"image": image_b64}},
        {
            "method": "post",
            "path": "/slide_match",
            "json": {
                "target_image": slide_target_b64,
                "background_image": slide_background_b64,
                "simple_target": True,
            },
        },
        {
            "method": "post",
            "path": "/slide_match",
            "json": {
                "target_image": slide_target_b64,
                "background_image": slide_background_b64,
                "simple_target": False,
            },
        },
        {
            "method": "post",
            "path": "/slide_comparison",
            "json": {
                "target_image": cmp_target_b64,
                "background_image": cmp_background_b64,
            },
        },
        {
            "method": "post",
            "path": "/mcp",
            "json": {
                "jsonrpc": "2.0",
                "id": 0,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "ddddocr",
                        "version": "1.0.0",
                    },
                },
            },
        },
        {
            "method": "post",
            "path": "/mcp",
            "json": {
                "jsonrpc": "2.0",
                "id": 0,
                "method": "tools/list",
                "params": {},
            },
        },
        {
            "method": "post",
            "path": "/mcp",
            "json": {
                "jsonrpc": "2.0",
                "id": 0,
                "method": "tools/call",
                "params": {
                    "name": "ocr",
                    "arguments": {"image": image_b64, "color_filter": "green"},
                },
            },
        },
    ]

    def build_curl(method, url, json_data=None):
        def replace_image(data):
            if isinstance(data, dict):
                new_data = {}
                for k, v in data.items():
                    if "image" in k.lower():
                        new_data[k] = "base64 image"
                    else:
                        new_data[k] = replace_image(v)
                return new_data
            elif isinstance(data, list):
                return [replace_image(item) for item in data]
            else:
                return data

        parts = []
        parts.append(f'curl -X {method.upper()} "{url}"')

        if json_data is not None:
            parts.append('-H "Content-Type: application/json"')
            data = replace_image(json_data)
            json_str = json.dumps(data, ensure_ascii=False)
            parts.append(f"-d '{json_str}'")

        return "\n       ".join(parts)

    for route in routes:
        url = f"{base_url}{route['path']}"
        curl_cmd = build_curl(route["method"], url, route.get("json"))
        try:
            if route["method"].lower() == "get":
                resp = requests.get(url)
            else:
                resp = requests.post(url, json=route["json"])

            print(f"--> {resp.status_code} {route['method'].upper()} {route['path']}\n")
            print(f"  {curl_cmd}\n")
            print(f"  {resp.text}\n")
        except Exception as e:
            print(f"--> ERROR {route['method'].upper()} {route['path']}\n")
            print(f"  {curl_cmd}\n")
            print(f"  {e}\n")


if __name__ == "__main__":
    test_routes()
