import requests
import base64

host = "http://127.0.0.1:9898"
file = open('./image/3.png', 'rb').read()

# ===============================================================
# 测试 jsonp，只能使用 b64，不能使用 file
# ===============================================================

api_url = f"{host}/ocr/b64/text" 
resp = requests.get(api_url, params = {
  "callback": "handle",
  "image": base64.b64encode(file).decode(),
})

print(f"jsonp, api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr/b64/json" 
resp = requests.get(api_url, params = {
  "callback": "handle",
  "image": base64.b64encode(file).decode(),
})

print(f"jsonp, api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试 ocr
# ===============================================================

api_url = f"{host}/ocr/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试 old
# ===============================================================

api_url = f"{host}/old/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试 ocr 概率
# ===============================================================

api_url = f"{host}/ocr_probability/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr_probability/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr_probability/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/ocr_probability/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试 old 概率
# ===============================================================

api_url = f"{host}/old_probability/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old_probability/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old_probability/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/old_probability/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试目标检测
# ===============================================================

file = open('./image/5.jpg', 'rb').read()
api_url = f"{host}/det/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/det/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/det/b64/text"
resp = requests.post(api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/det/b64/json"
resp = requests.post(api_url, json={'image': base64.b64encode(file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试滑块，算法 1
# ===============================================================

target_file = open('image/a.png', 'rb').read()
bg_file = open('image/b.png', 'rb').read()

api_url = f"{host}/match/file/text"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/match/file/json"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/match/b64/text"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/match/b64/json"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试滑块，算法 2
# ===============================================================

target_file = open('image/a.png', 'rb').read()
bg_file = open('image/b.png', 'rb').read()

api_url = f"{host}/simple_match/file/text"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/simple_match/file/json"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/simple_match/b64/text"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/simple_match/b64/json"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

# ===============================================================
# 测试坑位匹配
# ===============================================================

target_file = open('image/c.jpg', 'rb').read()
bg_file = open('image/d.jpg', 'rb').read()

api_url = f"{host}/compare/file/text"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/compare/file/json"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/compare/b64/text"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")

api_url = f"{host}/compare/b64/json"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"api_url={api_url}, resp.text={resp.text}")
