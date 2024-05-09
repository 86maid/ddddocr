# 简介
ddddocr rust 版本。  

ocr_api_server rust 版本。

二进制版本，验证码识别，不依赖 opencv 库，跨平台运行。  

a simple OCR API server, very easy to deploy。

[<img alt="github" src="https://img.shields.io/badge/github-86maid/ddddocr- ?logo=github" height="20">](https://github.com/86maid/ddddocr)
[![Forks][forks-shield]](https://github.com/86maid/ddddocr)
[![Stargazers][stars-shield]](https://github.com/86maid/ddddocr)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/86maid/ddddocr/blob/master/LICENSE)

[forks-shield]: https://img.shields.io/github/forks/86maid/ddddocr?style=flat-square
[stars-shield]: https://img.shields.io/github/stars/86maid/ddddocr?style=flat-square

<p align="center">
  <a href="https://github.com/sml2h3/ddddocr">
    <img src="https://cdn.wenanzhe.com/img/logo.png!/crop/700x500a400a500" alt="Logo">
  </a>
  <p align="center">
    一个容易使用的通用验证码识别 rust 库
    <br />
    ·
    <a href="https://github.com/sml2h3/ddddocr/issues">报告Bug</a>
    ·
    <a href="https://github.com/sml2h3/ddddocr/issues">提出新特性</a>
  </p>
</p>

# 目录

- [简介](#简介)
- [目录](#目录)
- [环境支持](#环境支持)
- [安装步骤](#安装步骤)
  - [如果你不想从源代码构建，这里有编译好的二进制版本。](#如果你不想从源代码构建这里有编译好的二进制版本)
- [使用文档](#使用文档)
  - [OCR 识别](#ocr-识别)
    - [内容识别](#内容识别)
    - [旧模型](#旧模型)
    - [支持识别透明黑色 png 格式的图片，使用 png\_fix 参数](#支持识别透明黑色-png-格式的图片使用-png_fix-参数)
    - [参考例图](#参考例图)
  - [目标检测](#目标检测)
    - [参考例图](#参考例图-1)
  - [滑块匹配](#滑块匹配)
    - [算法1](#算法1)
    - [算法2](#算法2)
  - [OCR 概率输出](#ocr-概率输出)
  - [自定义 OCR 训练模型导入](#自定义-ocr-训练模型导入)
- [ocr\_api\_server 例子](#ocr_api_server-例子)
  - [运行方式](#运行方式)
  - [接口](#接口)
  - [接口测试例子，完整的测试请看 `test_api.py` 文件](#接口测试例子完整的测试请看-test_apipy-文件)
- [疑难杂症](#疑难杂症)

# 环境支持

| 系统             | CPU | GPU | 备注                                                                                 |
| ---------------- | --- | --- | ------------------------------------------------------------------------------------ |
| Windows 64位     | √   | ?   | 部分版本 Windows 需要安装 <a href="https://www.ghxi.com/yxkhj.html">vc运行库</a>     |
| Windows 32位     | √   | ?   | 部分版本 Windows 需要安装 <a href="https://www.ghxi.com/yxkhj.html">vc运行库</a>     |
| Linux 64 / ARM64 | √   | ?   |                                                                                      |
| Linux 32         | ×   | ?   |                                                                                      |
| Macos X64        | √   | ?   | M1/M2/M3 ... 芯片参考<a href="https://github.com/sml2h3/ddddocr/issues/67"> #67 </a> |

# 安装步骤

`lib.rs` 实现了 `ddddocr`。

`main.rs` 实现了 `ocr_api_server`。

`model` 目录是模型与字符集。

依赖本库 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master" }`  

开启 `cuda` 特性 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["cuda"] }`

开启 `cuda` 需要 `cuda 11` 的 `nvidia gpu` (不确定 `cuda 10` 是否有效)

请确保 onnxruntime 的动态链接库在程序的运行目录里面，否则运行会恐慌！

## 如果你不想从源代码构建，这里有编译好的[二进制版本](https://github.com/86maid/ddddocr/releases)。

# 使用文档

## OCR 识别

### 内容识别
主要用于识别单行文字，即文字部分占据图片的主体部分，例如常见的英数验证码等，本项目可以对中文、英文（随机大小写or通过设置结果范围圈定大小写）、数字以及部分特殊字符。

```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification().unwrap();
let res = ocr.classification(image, false).unwrap();
println!("{:?}", res);
```

### 旧模型

```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification_old().unwrap();
let res = ocr.classification(image, false).unwrap();
println!("{:?}", res);
```

### 支持识别透明黑色 png 格式的图片，使用 png_fix 参数

```
classification(image, true);
```

### 参考例图

<img src="https://cdn.wenanzhe.com/img/20210715211733855.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/78b7f57d-371d-4b65-afb2-d19608ae1892.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E5%9B%BE%E7%89%87_20211226142305.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E5%9B%BE%E7%89%87_20211226142325.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/2AMLyA_fd83e1f1800e829033417ae6dd0e0ae0.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/aabd_181ae81dd5526b8b89f987d1179266ce.jpg" alt="captcha" width="150">
<br />
<img src="https://cdn.wenanzhe.com/img/2bghz_b504e9f9de1ed7070102d21c6481e0cf.png" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/0000_z4ecc2p65rxc610x.jpg" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/2acd_0586b6b36858a4e8a9939db8a7ec07b7.jpg" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/2a8r_79074e311d573d31e1630978fe04b990.jpg" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/aftf_C2vHZlk8540y3qAmCM.bmp" alt="captcha" width="150">
<img src="https://cdn.wenanzhe.com/img/%E5%BE%AE%E4%BF%A1%E6%88%AA%E5%9B%BE_20211226144057.png" alt="captcha" width="150">

## 目标检测

```rust
let image = std::fs::read("target.png").unwrap();
let mut det = ddddocr::ddddocr_detection().unwrap();
let res = det.detection(image).unwrap();
println!("{:?}", res);
```

### 参考例图

![Test](https://cdn.wenanzhe.com/img/page1_1.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_2.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_3.jpg) 
![Test](https://cdn.wenanzhe.com/img/page1_4.jpg) 
![Test](https://cdn.wenanzhe.com/img/result.jpg) 
![Test](https://cdn.wenanzhe.com/img/result2.jpg) 
![Test](https://cdn.wenanzhe.com/img/result4.jpg) 

以上只是目前我能找到的点选验证码图片，做了一个简单的测试。

## 滑块匹配

算法非深度神经网络实现。

### 算法1
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

如果小图无过多背景部分，则可以使用 simple_slide_match，通常为 jpg 或者 bmp 格式的图片

```rust
let target_bytes = std::fs::read("target.png").unwrap();
let background_bytes = std::fs::read("background.png").unwrap();
let res = ddddocr::simple_slide_match(target_bytes, background_bytes).unwrap();
println!("{:?}", res);
```

### 算法2
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

## OCR 概率输出

为了提供更灵活的ocr结果控制与范围限定，项目支持对ocr结果进行范围限定。

可以通过在调用 `classification_probability` 返回全字符表的概率。

当然也可以通过 `set_ranges` 设置输出字符范围来限定返回的结果。

| 参数值 | 意义                                             |
| ------ | ------------------------------------------------ |
| 0      | 纯整数 0-9                                       |
| 1      | 纯小写字母 a-z                                   |
| 2      | 纯大写字母 A-Z                                   |
| 3      | 小写字母 a-z + 大写字母 A-Z                      |
| 4      | 小写字母 a-z + 整数 0-9                          |
| 5      | 大写字母 A-Z + 整数 0-9                          |
| 6      | 小写字母 a-z + 大写字母A-Z + 整数0-9             |
| 7      | 默认字符库 - 小写字母a-z - 大写字母A-Z - 整数0-9 |

如果值为 string 类型，请传入一段不包含空格的文本，其中的每个字符均为一个待选词，例如：`"0123456789+-x/="`

```rust
let image = std::fs::read("image.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification().unwrap();

// 数字 3 对应枚举 CharsetRange::LowercaseUppercase
// ocr.set_ranges(3);

// 自定义字符集
ocr.set_ranges("0123456789+-x/=");

let result = ocr.classification_probability(image, false).unwrap();

// 哦呀，看来数据有点儿太多了，小心卡死哦！
println!("概率: {}", result.json());

println!("识别结果: {}", result.get_text());
```

## 自定义 OCR 训练模型导入

支持导入 [dddd_trainer](https://github.com/sml2h3/dddd_trainer) 训练后的自定义模型。

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

# ocr_api_server 例子

## 运行方式
```cmd
Usage: ddddocr.exe [OPTIONS]

Options:
  -a, --address <ADDRESS>
          监听地址 [default: 127.0.0.1]
  -p, --port <PORT>
          监听端口 [default: 9898]
  -f, --full
          开启所有选项
      --jsonp
          开启跨域，需要一个 query 指定回调函数的名字，不能使用 file (multipart) 传递参数， 例如 http://127.0.0.1:9898/ocr/b64/text?callback=handle&image=xxx
      --ocr
          开启内容识别，支持新旧模型共存
      --old
          开启旧版模型内容识别，支持新旧模型共存
      --det
          开启目标检测
      --ocr-probability <OCR_PROBABILITY>
          开启内容概率识别，支持新旧模型共存，只能使用官方模型， 如果参数是 0 到 7，对应内置的字符集， 如果参数空字符串，表示默认字符集， 除此之外的参数，表示自定义字符集，例如 "0123456789+-x/="
      --old-probability <OLD_PROBABILITY>
          开启旧版模型内容概率识别，支持新旧模型共存，只能使用官方模型， 如果参数是 0 到 7，对应内置的字符集， 如果参数空字符串，表示默认字符集， 除此之外的参数，表示自定义字符集，例如 "0123456789+-x/="
      --ocr-path <OCR_PATH>
          内容识别模型以及字符集路径， 通过哈希值判断是否为自定义模型， 使用自定义模型会使 old 选项失效， 路径 model/common 对应模型 model/common.onnx 和字符集 model/common.json [default: model/common]
      --det-path <DET_PATH>
          目标检测模型路径 [default: model/common_det.onnx]
      --slide-match
          开启滑块识别
      --simple-slide-match
          开启简单滑块识别
      --slide-compare
          开启坑位识别
  -h, --help
          Print help
```

## 接口
测试是否启动成功，可以通过直接 `GET/POST` 访问 `http://{host}:{port}/ping` 来测试，如果返回 `pong` 则启动成功。

```
http://{host}:{port}/{opt}/{img_type}/{ret_type}

opt:
  ocr               内容识别
  old               旧版模型内容识别
  det               目标检测
  ocr_probability   内容概率识别
  old_probability   旧版模型内容概率识别
  match             滑块匹配
  simple_match      简单滑块匹配
  compare           坑位匹配

img_type:
  file          文件，即 multipart/form-data
  b64           base64，即 {"a": encode(bytes), "b": encode(bytes)}

ret_type:
  json          json，成功 {"status": 200, "result": object}，失败 {"status": 404, "msg": "失败原因"}
  text          文本，失败返回空文本
```

## 接口测试例子，完整的测试请看 `test_api.py` 文件

```python
import requests
import base64

host = "http://127.0.0.1:9898"
file = open('./image/3.png', 'rb').read()

# 测试 jsonp，只能使用 b64，不能使用 file
api_url = f"{host}/ocr/b64/text" 
resp = requests.get(api_url, params = {
  "callback": "handle",
  "image": base64.b64encode(file).decode(),
})
print(f"jsonp, api_url={api_url}, resp.text={resp.text}")

# 测试 ocr
api_url = f"{host}/ocr/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")
```

# 疑难杂症
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

如果你在 linux 编译失败，尝试使用 `apt install binutils`，然后 `cargo clean`，再重新编译。

如果你 linux 和 osx 版本编译失败，尝试使用 `cargo zigbuild`，这将使用 zig 的链接器，本人亲测，有奇效！

其他疑难杂症请访问 [onnxruntime-rs](https://github.com/nbigaouette/onnxruntime-rs)。
