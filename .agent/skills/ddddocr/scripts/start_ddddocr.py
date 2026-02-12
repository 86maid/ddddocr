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
import time
import stat
from pathlib import Path

import requests


def get_platform_info():
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "windows":
        arch = (
            "x64"
            if machine in ("amd64", "x86_64")
            else "x86" if machine in ("x86", "i386") else None
        )
        if arch == "x64":
            return "x86_64-pc-windows-msvc-inline.zip"
        elif arch == "x86":
            return "i686-pc-windows-msvc-inline.zip"
    elif system == "linux":
        arch = (
            "arm64"
            if machine in ("aarch64", "arm64")
            else "x64" if machine in ("amd64", "x86_64") else None
        )
        if arch == "arm64":
            return "aarch64-unknown-linux-gnu-inline.zip"
        elif arch == "x64":
            return "x86_64-unknown-linux-musl-inline.zip"
    elif system == "darwin":
        arch = (
            "arm64"
            if machine in ("aarch64", "arm64")
            else "x64" if machine in ("amd64", "x86_64") else None
        )
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


def get_version_file_path(cache_dir):
    return os.path.join(cache_dir, ".version")


def get_cached_version(cache_dir):
    version_file = get_version_file_path(cache_dir)
    if os.path.exists(version_file):
        try:
            with open(version_file, "r") as f:
                return f.read().strip()
        except:
            pass
    return None


def save_version(cache_dir, version):
    version_file = get_version_file_path(cache_dir)
    try:
        with open(version_file, "w") as f:
            f.write(version)
    except Exception as e:
        print(f"Warning: Could not save version info: {e}", file=sys.stderr)


def download_and_extract(url, dest_dir):
    temp_file_path = None
    try:
        with tempfile.NamedTemporaryFile(delete=False, suffix=".tmp") as temp_file:
            temp_file_path = temp_file.name

        with urllib.request.urlopen(url) as response:
            total_size = int(response.getheader("Content-Length", 0))
            block_size = 8192
            downloaded = 0
            start_time = time.time()

            with open(temp_file_path, "wb") as out_file:
                while True:
                    buffer = response.read(block_size)
                    if not buffer:
                        break
                    out_file.write(buffer)
                    downloaded += len(buffer)

                    if total_size > 0:
                        percent = downloaded / total_size * 100
                        elapsed = time.time() - start_time
                        speed = downloaded / 1024 / elapsed if elapsed > 0 else 0
                        print(
                            f"\rDownloading: {percent:6.2f}% "
                            f"({downloaded / 1024:.1f}KB/{total_size / 1024:.1f}KB) "
                            f"Speed: {speed:.1f} KB/s",
                            end="",
                            flush=True,
                        )
            print()

        if url.endswith(".zip"):
            with zipfile.ZipFile(temp_file_path, "r") as zip_ref:
                zip_ref.extractall(dest_dir)
        elif url.endswith(".tar.gz") or url.endswith(".tgz"):
            with tarfile.open(temp_file_path, "r:gz") as tar_ref:
                tar_ref.extractall(dest_dir)
    finally:
        if temp_file_path and os.path.exists(temp_file_path):
            os.unlink(temp_file_path)


def find_executable(directory):
    for root, dirs, files in os.walk(directory):
        for file in files:
            if platform.system().lower() == "windows" and file == "ddddocr.exe":
                return os.path.join(root, file)
            elif platform.system().lower() != "windows" and file == "ddddocr":
                return os.path.join(root, file)
    return None


def set_executable_permission(exe_path):
    if platform.system().lower() != "windows":
        try:
            os.chmod(
                exe_path,
                stat.S_IRWXU
                | stat.S_IRGRP
                | stat.S_IXGRP
                | stat.S_IROTH
                | stat.S_IXOTH,
            )
        except Exception as e:
            print(f"Warning: Could not set executable permission: {e}", file=sys.stderr)


def check_running(host, port):
    try:
        url = f"http://{host}:{port}/status"
        response = requests.get(url, timeout=1)
        return response.status_code == 200
    except requests.RequestException:
        return False


def wait_for_service(host, port, timeout=30):
    print("Waiting for service to start...", end="")
    for _ in range(timeout):
        if check_running(host, port):
            print(" OK")
            return True
        time.sleep(1)
        print(".", end="", flush=True)
    print(" Timeout")
    return False


def start_ddddocr(exe_path, address="127.0.0.1", port=8000):
    args = [
        exe_path,
        "--address",
        f"{address}:{port}",
        "--ocr",
        "--det",
        "--slide",
        "--mcp",
    ]

    if platform.system().lower() == "windows":
        process = subprocess.Popen(
            args, creationflags=subprocess.CREATE_NEW_PROCESS_GROUP
        )
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
        print(
            f"Error: Unsupported platform {platform.system()} {platform.machine()}",
            file=sys.stderr,
        )
        return 1

    cache_dir = os.path.join(os.path.expanduser("~"), ".ddddocr_cache")
    os.makedirs(cache_dir, exist_ok=True)

    url, latest_version = get_latest_release_info(filename)

    if not url:
        print(f"Error: Could not find release for {filename}", file=sys.stderr)
        return 1

    exe_path = find_executable(cache_dir)
    cached_version = get_cached_version(cache_dir) if exe_path else None

    need_download = False
    if not exe_path or cached_version != latest_version:
        need_download = True
        action = (
            "Downloading"
            if not exe_path
            else f"Updating ({cached_version} -> {latest_version})"
        )
        print(f"{action} DDDDOCR: {filename}...")

    if need_download:
        print(f"Downloading from: {url}")
        temp_dir = tempfile.mkdtemp()
        try:
            download_and_extract(url, temp_dir)
            new_exe_path = find_executable(temp_dir)
            if not new_exe_path:
                raise Exception("Executable not found after extraction")
            set_executable_permission(new_exe_path)
            shutil.rmtree(cache_dir, ignore_errors=True)
            shutil.move(temp_dir, cache_dir)
            exe_path = find_executable(cache_dir)
            set_executable_permission(exe_path)
            save_version(cache_dir, latest_version)
            print("Download and update completed.")
        except Exception as e:
            print(f"Error during download/update: {e}", file=sys.stderr)
            shutil.rmtree(temp_dir, ignore_errors=True)
            if cached_version and os.path.exists(cache_dir):
                exe_path = find_executable(cache_dir)
            else:
                return 1
    else:
        print(f"Using cached DDDDOCR: v{latest_version}")
        set_executable_permission(exe_path)

    if not exe_path:
        print("Error: Could not find ddddocr executable", file=sys.stderr)
        return 1

    print(f"Executable: {exe_path}")
    print(f"Starting DDDDOCR service on {address}:{port}...")
    print(f"Features: ocr, det, slide, mcp")

    process = start_ddddocr(exe_path, address, port)

    print(f"Service started with PID: {process.pid}")
    print(f"MCP endpoint: http://{address}:{port}/mcp")
    print(f"API documentation: http://{address}:{port}/docs")

    if not wait_for_service(address, port):
        print("Warning: Service did not start within expected time", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
