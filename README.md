# 简介
[中文](./README.md) | [English](./README_EN.md)

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
  - [还可以使用配置好的 Github Action 进行构建。](#还可以使用配置好的-github-action-进行构建)
- [使用文档](#使用文档)
  - [OCR 识别](#ocr-识别)
    - [内容识别](#内容识别)
    - [旧模型](#旧模型)
    - [支持识别透明黑色 png 格式的图片，使用 png\_fix 参数](#支持识别透明黑色-png-格式的图片使用-png_fix-参数)
    - [颜色过滤](#颜色过滤)
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
  - [运行例子](#运行例子)
  - [API 说明](#api-说明)
  - [API 测试例子，完整的测试请看 `test_api.py` 文件](#api-测试例子完整的测试请看-test_apipy-文件)
  - [MCP 协议支持](#mcp-协议支持)
    - [工具调用请求](#工具调用请求)
    - [工具调用响应](#工具调用响应)
- [疑难杂症](#疑难杂症)

# 环境支持

| 系统             | CPU | GPU | 备注                                                                                                 |
| ---------------- | --- | --- | ---------------------------------------------------------------------------------------------------- |
| Windows 64位     | √   | ?   | 部分版本 Windows 需要安装 <a href="https://www.ghxi.com/yxkhj.html">vc 运行库</a>                    |
| Windows 32位     | √   | ?   | 不支持静态链接，部分版本 Windows 需要安装 <a href="https://www.ghxi.com/yxkhj.html">vc 运行库</a>    |
| Linux 64 / ARM64 | √   | ?   | 可能需要升级 glibc 版本， <a href=https://www.cnblogs.com/fireinstone/p/18169273>升级 glibc 版本</a> |
| Linux 64 / MUSL  | √   | ?   | 不需要 glibc，静态链接                                                                               |
| Linux 32         | ×   | ?   |                                                                                                      |
| Macos X64        | √   | ?   | M1/M2/M3 ... 芯片参考<a href="https://github.com/sml2h3/ddddocr/issues/67"> #67 </a>                 |

# 安装步骤

`lib.rs` 实现了 `ddddocr`。

`main.rs` 实现了 `ocr_api_server`。

`model` 目录是模型与字符集。

依赖本库 `ddddocr = {git = "https://github.com/86maid/ddddocr.git", branch = "master"}`  

开启 `cuda` 特性 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["cuda"] }`

支持静态和动态链接，默认使用静态链接，构建时将会自动下载链接库，请设置好代理，`cuda` 特性不支持静态链接（会自己下载动态链接库）。

如有更多问题，请跳转至[疑难杂症](#疑难杂症)部分。

## 如果你不想从源代码构建，这里有编译好的[二进制版本](https://github.com/86maid/ddddocr/releases)。

## 还可以使用配置好的 [Github Action](https://github.com/86maid/ddddocr/tree/master/.github/workflows) 进行构建。

# 使用文档

## OCR 识别

### 内容识别
主要用于识别单行文字，即文字部分占据图片的主体部分，例如常见的英数验证码等，本项目可以对中文、英文（随机大小写or通过设置结果范围圈定大小写）、数字以及部分特殊字符。

```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification().unwrap();
let res = ocr.classification(image).unwrap();
println!("{:?}", res);
```

### 旧模型

```rust
let image = std::fs::read("target.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification_old().unwrap();
let res = ocr.classification(image).unwrap();
println!("{:?}", res);
```

### 支持识别透明黑色 png 格式的图片，使用 png_fix 参数

```
classification_with_png_fix(image, true);
```

### 颜色过滤

支持以下预设颜色：red（红色）、blue（蓝色）、green（绿色）、yellow（黄色）、orange（橙色）、purple（紫色）、cyan（青色）、black（黑色）、white（白色）、gray（灰色）。

```rust
let ddddocr = ddddocr_classification().unwrap();

// 只保留绿色
println!(
    "{}",
    ddddocr
        .classification_with_filter(include_bytes!("../image/4.png"), "green")
        .unwrap()
);

// 只保留红色和绿色
println!(
    "{}",
    ddddocr
        .classification_with_filter(include_bytes!("../image/4.png"), ["red", "green"])
        .unwrap()
);

// HSV 范围，每个元素是一个 (min_hsv, max_hsv) 的元组。
println!(
    "{}",
    ddddocr
        .classification_with_filter(
            include_bytes!("../image/4.png"),
            [((40, 50, 50), (80, 255, 255))]
        )
        .unwrap()
);
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

为了提供更灵活的 ocr 结果控制与范围限定，项目支持对ocr结果进行范围限定。

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

// 数字 3 对应枚举 CharsetRange::LowercaseUppercase，不用写枚举
// ocr.set_ranges(3);

// 设置全局字符集
ocr.set_ranges("0123456789+-x/=");

// 或者，单次识别的字符集
// ocr.classification_probability_with_ranges(image, "0123456789+-x/=");

let result = ocr.classification_probability(image).unwrap();

println!("识别结果: {}", result.get_text());
println!("识别可信度: {}", result.get_confidence());

// 哦呀，看来数据有点儿太多了，小心卡死哦！
println!("概率: {}", result.json());
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
```sh
Usage: ddddocr.exe [OPTIONS]

Options:
      --address <ADDRESS>
          监听地址。 [default: 0.0.0.0:8000]
      --mcp
          mcp 协议支持，与 only_mcp 互斥。
      --only-mcp
          仅开启 mcp 协议，不开启普通路由，与 mcp 互斥。
      --ocr
          开启内容识别，与 old 互斥。
      --old
          开启旧版模型内容识别，与 ocr 互斥。
      --det
          开启目标检测。
      --slide
          开启滑块和坑位识别。
      --ocr-charset-range <OCR_CHARSET_RANGE>
          全局默认字符集，用于概率识别， 如果 API 未提供字符集，则使用此参数， 当值为 0~7 时，表示选择内置字符集， 其他值表示自定义字符集，例如 "0123456789+-x/="， 如果未设置，则使用完整字符集，不做限制。
      --ocr-path <OCR_PATH>
          内容识别模型以及字符集路径， 如果你开启了 features 的 inline-model 选项（默认开启），则不用管这个选项，除非你想使用自定义模型， 模型 model/common.onnx 和字符集 model/common.json 要同名。 [default: model/common.onnx]
      --det-path <DET_PATH>
          目标检测模型路径， 如果你开启了 features 的 inline-model 选项（默认开启），则不用管这个选项，除非你想使用自定义模型。 [default: model/common_det.onnx]
      --acme <ACME>
          输入你的域名，自动获取 SSL 证书， 即 https 的支持。
  -h, --help
          Print help
```

## 运行例子
```sh
# 启动所有功能
ddddocr.exe --address 0.0.0.0:8000 --ocr --det --slide

# 查看所有选项
ddddocr.exe --help
```

## API 说明

| 端点                | 方法 | 说明             |
| ------------------- | ---- | ---------------- |
| `/ocr`              | POST | 执行OCR识别      |
| `/det`              | POST | 执行目标检测     |
| `/slide-match`      | POST | 滑块匹配算法     |
| `/slide-comparison` | POST | 滑块比较算法     |
| `/status`           | GET  | 获取当前服务状态 |
| `/docs`             | GET  | Swagger UI 文档  |

## API 测试例子，完整的测试请看 `test_api.py` 文件

```python
--> 200 GET /status

  curl -X GET "http://127.0.0.1:8000/status"

  {"code":200,"msg":"success","data":{"service_status":"running","enabled_features":["ocr","det","slide"]}}

--> 200 POST /ocr

  curl -X POST "http://127.0.0.1:8000/ocr"  
       -H "Content-Type: application/json"  
       -d '{"image": "base64 image"}'       

  {"code":200,"msg":"success","data":{"text":"九乘六等于？","probability":null}}

--> 200 POST /det

  curl -X POST "http://127.0.0.1:8000/det"
       -H "Content-Type: application/json"
       -d '{"image": "base64 image"}'

  {"code":200,"msg":"success","data":{"bboxes":[[80,3,98,21],[56,6,76,25],[31,7,51,26],[2,2,21,22],[100,0,127,18]]}}

--> 200 POST /slide-match

  curl -X POST "http://127.0.0.1:8000/slide-match"
       -H "Content-Type: application/json"
       -d '{"target_image": "base64 image", "background_image": "base64 image", "simple_target": true}'

  {"code":200,"msg":"success","data":{"target":[215,45,261,91],"target_x":0,"target_y":45}}

--> 200 POST /slide-comparison

  curl -X POST "http://127.0.0.1:8000/slide-comparison"
       -H "Content-Type: application/json"
       -d '{"target_image": "base64 image", "background_image": "base64 image"}'

  {"code":200,"msg":"success","data":{"x":144,"y":76}}
```
## MCP 协议支持

本项目支持 MCP（Model Context Protocol）协议，使 AI Agent 能够直接调用 ddddocr 服务。

版本：2025-11-25

端点：`POST /mcp`

方法：`initialize` `tools/list` `tools/call`  

### 工具调用请求

```json
{
    "jsonrpc": "2.0",
    "id": 0,
    "method": "tools/call",
    "params": {
        "name": "ocr",
        "arguments": {"image": "image_b64", "color_filter": "green"},
    },
}
```
### 工具调用响应

```json
{
  "jsonrpc": "2.0",
  "id": 0,
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

# 疑难杂症

强烈推荐用 [Github Action](https://github.com/86maid/ddddocr/tree/master/.github/workflows) 进行构建。

关于使用 `cuda` 的问题。

`cuda` 和 `cuDNN` 都需要安装好。

`CUDA 12` 构建需要 `cuDNN 9.x`。

`CUDA 11` 构建需要 `cuDNN 8.x`。

不确定 `cuda 10` 是否有效。

默认使用静态链接，构建时将会自动下载链接库，请设置好代理，`cuda` 特性不支持静态链接（会自己下载动态链接库）。

如果要指定静态链接库的路径，可以设置环境变量 `ORT_LIB_LOCATION`，设置后将不会自动下载链接库。

例如，库路径为 `onnxruntime\build\Windows\Release\Release\onnxruntime.lib`，则 `ORT_LIB_LOCATION` 设置为 `onnxruntime\build\Windows\Release`。

默认开启 `download-binaries` 特性，自动下载链接库。

下载不了大部分是网络问题，开启代理后，记得重启 vscode，重启终端，以便代理能够使用 https_proxy 环境变量。

自动下载的链接库存放在 `C:\Users\<用户名>\AppData\ort.pyke.io`。

开启动态链接特性 `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["load-dynamic"] }`

开启 `load-dynamic` 特性后，可以使用 `Ddddocr::set_onnxruntime_path` 指定 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.18.1) 动态链接库的路径。

开启 `load-dynamic` 特性后，构建时将不会自动下载 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.18.1) 链接库。

请手动下载 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.18.1) 链接库，并将其放置在程序运行目录下（或系统 API 目录），这样无需再次调用 `Ddddocr::set_onnxruntime_path`。

windows 静态链接失败，请安装 vs2022。

linux musl 要用 docker 编译。

linux x86-64 静态链接失败，请安装 gcc11 和 g++11，ubuntu ≥ 20.04。

linux arm64 静态链接失败，需要 glibc ≥ 2.35 （Ubuntu ≥ 22.04）。

macOS 静态链接失败，需要 macOS ≥ 10.15。

cuda 在执行 `cargo test` 的时候可能会 `painc (exit code: 0xc000007b)`，这是因为自动生成的动态链接库是在 `target/debug` 目录下，需要手动复制到 `target/debug/deps` 目录下（cuda 目前不支持静态链接）。

动态链接需要 1.18.x 版本的 [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.18.1)。

更多疑难杂症，请跳转至 [ort.pyke.io](https://ort.pyke.io/)。
