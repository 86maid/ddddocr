# 简介
ddddocr rust 版本，ocr_api_server rust 版本，二进制版本，验证码识别，不依赖 opencv 库，跨平台运行，a simple OCR API server, very easy to deploy。

`lib.rs` 实现了 `ddddocr`。

`main.rs` 实现了 `ocr_api_server`。


`model` 目录是模型与字符集。

依赖本库 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master" }`  

开启 `cuda` 特性 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["cuda"] }`

开启 `cuda` 需要 `cuda 11` 的 `nvidia gpu` (不确定 `cuda 10` 是否有效)

## 如果你不想从源代码构建，这里有编译好的[二进制版本](https://github.com/86maid/ddddocr/releases)。

支持使用 `ddddocr` 调用 `dddd_trainer` 训练后的自定义模型。

`dddd_trainer` 训练后会在 `models` 目录里导出 `charsets.json` 和 `onnx` 模型。

如下所示：
```rust
use ddddocr::*;

let mut ocr = Ddddocr::with_model_charset(
    "myproject_0.984375_139_13000_2022-02-26-15-34-13.onnx",
    "charsets.json",
)
.unwrap();
let image_bytes = std::fs::read("888e28774f815b01e871d474e5c84ff2.jpg").unwrap();
let res = ocr.classification(&image_bytes).unwrap();
println!("{:?}", res);
```

# ❗❗❗ 疑难杂症
在 windows 上依赖 [onnxruntime.dll](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)，需要将在压缩包的 `lib` 下的 [onnxruntime.dll](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1) 解压到运行目录或系统调用目录，`否则运行将会 panic (exit code: 0xc000007b)`。  

在 `linux` 上依赖 [libonnxruntime.so.1.8.1](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)，运行和构建的方式和 `windows` 平台大同小异。

运行时出现以下错误，请设置环境变量 `LD_LIBRARY_PATH` 为 `libonnxruntime.so.1.8.1` 所在的目录。

```
./ddddocr: error while loading shared libraries: libonnxruntime.so.1.8.1: cannot open shared object file: No such file or directory
```

在构建时有两种策略，可以设置环境变量 `ORT_STRATEGY` 的值为如下：
1. (默认) `download` 自动从网上下载 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)。
2. `system` 从本地安装 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)，此时要设置环境变量 `ORT_LIB_LOCATION` 的值为库的位置（解压），然后重启 VSCode 刷新环境变量。

在构建的时候，默认使用 `download` 策略，如果出现以下报错，这是因为自动下载依赖失败导致的，请设置好代理，或者手动下载 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)，并将其放在报错中所指 `into` 目录中（不要解压）。

```
error: failed to run custom build command for `onnxruntime-sys v0.0.14`

Caused by:
  process didn't exit successfully: `C:\Users\XChuang233\Desktop\ddddocr-rust\ddddocr\target\debug\build\onnxruntime-sys-d30ec19d280a0792\build-script-build` (exit code: 101)
  --- stdout
  strategy: "unknown"
  cargo:rerun-if-changed=C:\Users\XChuang233\Desktop\ddddocr-rust\ddddocr\target\debug\build\onnxruntime-sys-1098f02db763c8b2\out\onnxruntime-win-x64-1.8.1.zip
  Creating directory "C:\\Users\\XChuang233\\Desktop\\ddddocr-rust\\ddddocr\\target\\debug\\build\\onnxruntime-sys-1098f02db763c8b2\\out"
  Downloading https://github.com/microsoft/onnxruntime/releases/download/v1.8.1/onnxruntime-win-x64-1.8.1.zip into C:\Users\XChuang233\Desktop\ddddocr-rust\ddddocr\target\debug\build\onnxruntime-sys-1098f02db763c8b2\out\onnxruntime-win-x64-1.8.1.zip
```

注意，如果你开启了 `cuda` 特性，则要下载 `gpu` 版本的 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.8.1)，可以设置 `ORT_STRATEGY` 的值为 `download ORT_USE_CUDA=1` 自动下载依赖。

强烈建议使用 `system` 策略，不然编译半天。

其他疑难杂症请访问 [onnxruntime-rs](https://github.com/nbigaouette/onnxruntime-rs)。

# 滑块部分
算法非深度神经网络实现。

## 算法1
小滑块为单独的png图片，背景是透明图，如下图：

![Test](https://cdn.wenanzhe.com/img/b.png) 

然后背景为带小滑块坑位的，如下图：

![Test](https://cdn.wenanzhe.com/img/a.png) 

```rust
let target_bytes = std::fs::read("target.png").unwrap();
let background_bytes = std::fs::read("background.png").unwrap();
let res = ddddocr::slide_match(target_bytes, background_bytes).unwrap();
println!("{:?}", res);
```
*提示：如果小图无过多背景部分，则可以添加simple_target参数， 通常为jpg或者bmp格式的图片*
```rust
let target_bytes = std::fs::read("target.png").unwrap();
let background_bytes = std::fs::read("background.png").unwrap();
let res = ddddocr::simple_slide_match(target_bytes, background_bytes).unwrap();
println!("{:?}", res);
```

## 算法2
一张图为带坑位的原图，如下图：

![Test](https://cdn.wenanzhe.com/img/bg.jpg) 

一张图为原图，如下图：

![Test](https://cdn.wenanzhe.com/img/fullpage.jpg) 

```rust
let target_bytes = std::fs::read("target.png").unwrap();
let background_bytes = std::fs::read("background.png").unwrap();
let res = ddddocr::slide_comparison(target_bytes, background_bytes).unwrap();
println!("{:?}", res);
```

# OCR 部分

## 内容识别
```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification().unwrap();
let res = ocr.classification(image).unwrap();
println!("{:?}", res);
```

## 旧模型
```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification_old().unwrap();
let res = ocr.classification(image).unwrap();
println!("{:?}", res);
```

OCR部分应该已经有很多人做了测试，在这里就放一部分网友的测试图片。

![Test](https://cdn.wenanzhe.com/img/20210715211733855.png) 
![Test](https://cdn.wenanzhe.com/img/78b7f57d-371d-4b65-afb2-d19608ae1892.png) 
![Test](https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E5%9B%BE%E7%89%87_20211226142305.png) 
![Test](https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E5%9B%BE%E7%89%87_20211226142325.png) 
![Test](https://cdn.wenanzhe.com/img/2AMLyA_fd83e1f1800e829033417ae6dd0e0ae0.png) 
![Test](https://cdn.wenanzhe.com/img/aabd_181ae81dd5526b8b89f987d1179266ce.jpg) 
![Test](https://cdn.wenanzhe.com/img/2bghz_b504e9f9de1ed7070102d21c6481e0cf.png) 
![Test](https://cdn.wenanzhe.com/img/0000_z4ecc2p65rxc610x.jpg) 
![Test](https://cdn.wenanzhe.com/img/2acd_0586b6b36858a4e8a9939db8a7ec07b7.jpg) 
![Test](https://cdn.wenanzhe.com/img/2a8r_79074e311d573d31e1630978fe04b990.jpg) 
![Test](https://cdn.wenanzhe.com/img/aftf_C2vHZlk8540y3qAmCM.bmp) 
![Test](https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E6%88%AA%E5%9B%BE_20211226144057.png) 

等等更多图片等你测试哟~

# 目标检测
```rust
let image = std::fs::read("target.png").unwrap();
let mut det = ddddocr::ddddocr_detection().unwrap();
let res = det.detection(image).unwrap();
println!("{:?}", res);
```

举些例子：

![Test](https://cdn.wenanzhe.com/img/page1_1.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_2.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_3.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_4.jpg) 
![Test](https://cdn.wenanzhe.com/img/result.jpg) 
![Test](https://cdn.wenanzhe.com/img/result2.jpg) 
![Test](https://cdn.wenanzhe.com/img/result4.jpg) 

以上只是目前我能找到的点选验证码图片，做了一个简单的测试。

# ocr_api_server 例子

## 运行方式
```cmd
Usage: ddddocr.exe [OPTIONS]

Options:
  -a, --address <ADDRESS>      监听地址 [default: 127.0.0.1]
  -p, --port <PORT>            监听端口 [default: 9898]
  -f, --full                   开启所有选项
      --ocr                    开启内容识别，支持新旧模型共存
      --old                    开启旧版模型内容识别，支持新旧模型共存
      --det                    开启目标检测
      --ocr-path <OCR_PATH>    内容识别模型以及字符集路径， 通过哈希值判断是否为自定义模型， 使用自定义模型会使 old 选项失效， 路径 model/common 对应模型 model/common.onnx 和字符集 model/common.json [default: model/common]
      --det-path <DET_PATH>    目标检测模型路径 [default: model/common_det.onnx]
      --slide-match            开启滑块识别
      --slide-compare          开启坑位识别
      --ocr-count <OCR_COUNT>  创建多个内容识别实例，提高并发的性能 [default: 1]
      --old-count <OLD_COUNT>  创建多个旧版模型内容识别实例，提高并发的性能 [default: 1]
      --det-count <DET_COUNT>  创建多个目标检测实例，提高并发的性能 [default: 1]
  -h, --help                   Print help
```

## 接口
测试是否启动成功，可以通过直接 `GET/POST` 访问 `http://{host}:{port}/ping` 来测试，如果返回 `pong`则启动成功。

```
http://{host}:{port}/{opt}/{img_type}/{ret_type}

opt:
  ocr       内容识别
  old       旧版模型内容识别
  det       目标检测
  match     滑块匹配
  compare   坑位匹配

img_type:
  file      文件，即 multipart/form-data
  b64       base64，即 {"a": encode(bytes), "b": encode(bytes)}

ret_type:
  json      json，成功 {"status": 200, "result": object}，失败 {"status": 404, "msg": "失败原因"}
  text      文本，失败返回空文本
```

### 具体请看 test_api.py 文件
```python
import base64
import requests

host = "http://127.0.0.1:9898"
file = open('image/3.png', 'rb').read()

api_url = f"{host}/ocr/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/ocr/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/ocr/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/ocr/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/old/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/old/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/old/b64/text"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/old/b64/json"
resp = requests.post(
    api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/det/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/det/file/json"
resp = requests.post(api_url, files={'image': file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/det/b64/text"
resp = requests.post(api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/det/b64/json"
resp = requests.post(api_url, json={'image': base64.b64encode(file).decode()})
print(f"{api_url=}, {resp.text=}")


# =============================================================
# =============================================================
# =============================================================

target_file = open('image/a.png', 'rb').read()
bg_file = open('image/b.png', 'rb').read()

api_url = f"{host}/match/file/text"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/match/file/json"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/match/b64/text"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/match/b64/json"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"{api_url=}, {resp.text=}")


# =============================================================
# =============================================================
# =============================================================

target_file = open('image/c.jpg', 'rb').read()
bg_file = open('image/d.jpg', 'rb').read()

api_url = f"{host}/compare/file/text"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/compare/file/json"
resp = requests.post(
    api_url, files={'target': target_file, 'background': bg_file})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/compare/b64/text"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"{api_url=}, {resp.text=}")

api_url = f"{host}/compare/b64/json"
resp = requests.post(
    api_url, json={'target': base64.b64encode(target_file).decode(), 'background': base64.b64encode(bg_file).decode()})
print(f"{api_url=}, {resp.text=}")
```
