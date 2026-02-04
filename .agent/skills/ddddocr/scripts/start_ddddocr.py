#!/usr/bin/env python3
import os
import platform
import subprocess
import sys
import urllib.request
import tarfile
import zipfile
import tempfile
import shutil
from pathlib import Path


def get_platform_info():
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "windows":
        arch = "x64" if machine in ("amd64", "x86_64") else "x86" if machine in ("x86", "i386") else None
        if arch == "x64":
            return "x86_64-pc-windows-msvc-inline.zip"
        elif arch == "x86":
            return "i686-pc-windows-msvc-inline.zip"
    elif system == "linux":
        arch = "arm64" if machine in ("aarch64", "arm64") else "x64" if machine in ("amd64", "x86_64") else None
        if arch == "arm64":
            return "aarch64-unknown-linux-gnu-inline.zip"
        elif arch == "x64":
            return "linux-x86_64-inline.zip"
    elif system == "darwin":
        arch = "arm64" if machine in ("aarch64", "arm64") else "x64" if machine in ("amd64", "x86_64") else None
        if arch == "arm64":
            return "aarch64-apple-darwin-inline.zip"
        elif arch == "x64":
            return "macos-x86_64-inline.zip"
    return None


def get_latest_release_info(filename):
    releases_url = "https://api.github.com/repos/86maid/ddddocr/releases/latest"
    
    try:
        with urllib.request.urlopen(releases_url) as response:
            data = response.read().decode()
            import json
            release = json.loads(data)
            
            version = release.get("tag_name", "")
            
            for asset in release.get("assets", []):
                name = asset.get("name", "")
                if name == filename:
                    return asset.get("browser_download_url"), version
    except Exception as e:
        print(f"Error fetching release info: {e}", file=sys.stderr)
    
    return None, None


def get_latest_release_url(filename):
    url, _ = get_latest_release_info(filename)
    return url


def get_version_file_path(cache_dir):
    return os.path.join(cache_dir, ".version")


def get_cached_version(cache_dir):
    version_file = get_version_file_path(cache_dir)
    if os.path.exists(version_file):
        try:
            with open(version_file, 'r') as f:
                return f.read().strip()
        except:
            pass
    return None


def save_version(cache_dir, version):
    version_file = get_version_file_path(cache_dir)
    try:
        with open(version_file, 'w') as f:
            f.write(version)
    except Exception as e:
        print(f"Warning: Could not save version info: {e}", file=sys.stderr)


def download_and_extract(url, dest_dir):
    temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=".tmp")
    
    try:
        with urllib.request.urlopen(url) as response:
            temp_file.write(response.read())
        
        if url.endswith(".zip"):
            with zipfile.ZipFile(temp_file.name, 'r') as zip_ref:
                zip_ref.extractall(dest_dir)
        elif url.endswith(".tar.gz") or url.endswith(".tgz"):
            with tarfile.open(temp_file.name, 'r:gz') as tar_ref:
                tar_ref.extractall(dest_dir)
    finally:
        temp_file.close()
        os.unlink(temp_file.name)


def find_executable(directory):
    for root, dirs, files in os.walk(directory):
        for file in files:
            if platform.system().lower() == "windows" and file == "ddddocr.exe":
                return os.path.join(root, file)
            elif platform.system().lower() != "windows" and file == "ddddocr":
                return os.path.join(root, file)
    return None


def check_running(host, port):
    try:
        import socket
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            result = s.connect_ex((host, port))
            return result == 0
    except:
        return False


def start_ddddocr(exe_path, address="127.0.0.1", port=8000):
    args = [exe_path, "--address", f"{address}:{port}", "--ocr", "--det", "--slide", "--mcp"]
    
    if platform.system().lower() == "windows":
        process = subprocess.Popen(args, creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
    else:
        process = subprocess.Popen(args, start_new_session=True)
    
    return process


def main():
    address = "127.0.0.1"
    port = 8000
    
    if check_running(address, port):
        print(f"DDDDOCR service already running on {address}:{port}")
        print(f"MCP endpoint: http://{address}:{port}/mcp")
        return 0
    
    filename = get_platform_info()
    if not filename:
        print(f"Error: Unsupported platform {platform.system()} {platform.machine()}", file=sys.stderr)
        return 1
    
    cache_dir = os.path.join(os.path.expanduser("~"), ".ddddocr_cache")
    exe_path = find_executable(cache_dir)
    
    url, latest_version = get_latest_release_info(filename)
    
    if not url:
        print(f"Error: Could not find release for {filename}", file=sys.stderr)
        return 1
    
    need_download = False
    
    if not exe_path:
        need_download = True
        print(f"Downloading DDDDOCR: {filename} (v{latest_version})...")
    else:
        cached_version = get_cached_version(cache_dir)
        if cached_version != latest_version:
            need_download = True
            print(f"Updating DDDDOCR: {filename} (v{cached_version} -> v{latest_version})...")
            shutil.rmtree(cache_dir)
            os.makedirs(cache_dir, exist_ok=True)
        else:
            print(f"Using cached DDDDOCR: v{latest_version}")
    
    if need_download:
        print(f"Downloading from: {url}")
        download_and_extract(url, cache_dir)
        save_version(cache_dir, latest_version)
        
        exe_path = find_executable(cache_dir)
        
        if not exe_path:
            print("Error: Could not find ddddocr executable after extraction", file=sys.stderr)
            return 1
    
    print(f"Executable: {exe_path}")
    print(f"Starting DDDDOCR service on {address}:{port}...")
    print(f"Features: ocr, det, slide, mcp")
    
    process = start_ddddocr(exe_path, address, port)
    
    print(f"Service started with PID: {process.pid}")
    print(f"MCP endpoint: http://{address}:{port}/mcp")
    print(f"API documentation: http://{address}:{port}/docs")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
