# Introduction

This project offers a Rust implementation of `ddddocr` and `ocr_api_server`. It provides a binary version for CAPTCHA recognition that doesn't rely on the OpenCV library, ensuring cross-platform compatibility. The goal is to deliver a simple OCR API server that's easy to deploy.

[<img alt="github" src="https://img.shields.io/badge/github-86maid/ddddocr- ?logo=github" height="20">](https://github.com/86maid/ddddocr)
[![Forks][forks-shield]](https://github.com/86maid/ddddocr)
[![Stargazers][stars-shield]](https://github.com/86maid/ddddocr)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/86maid/ddddocr/blob/master/LICENSE)

[forks-shield]: https://img.shields.io/github/forks/86maid/ddddocr?style=flat-square
[stars-shield]: https://img.shields.io/github/stars/86maid/ddddocr?style=flat-square

![Project Logo](https://cdn.wenanzhe.com/img/logo.png!/crop/700x500a400a500)

This is an easy-to-use, general-purpose CAPTCHA recognition library written in Rust. We encourage users to [report bugs](https://github.com/sml2h3/ddddocr/issues) and [suggest new features](https://github.com/sml2h3/ddddocr/issues).

# Table of Contents

- [Introduction](#introduction)
- [Table of Contents](#table-of-contents)
- [Supported Environments](#supported-environments)
- [Installation Steps](#installation-steps)
    - [From Source](#from-source)
    - [Precompiled Binaries](#precompiled-binaries)
    - [GitHub Actions](#github-actions)
- [Usage Documentation](#usage-documentation)
    - [OCR Recognition](#ocr-recognition)
        - [Content Recognition](#content-recognition)
        - [Legacy Model](#legacy-model)
        - [Transparent Black PNG Support](#transparent-black-png-support)
        - [Reference Images](#reference-images)
    - [Object Detection](#object-detection)
        - [Reference Images](#reference-images-1)
    - [Slider Matching](#slider-matching)
        - [Algorithm 1](#algorithm-1)
        - [Algorithm 2](#algorithm-2)
    - [OCR Probability Output](#ocr-probability-output)
    - [Custom OCR Model Import](#custom-ocr-model-import)
- [OCR API Server Example](#ocr-api-server-example)
    - [Usage](#usage)
    - [Endpoints](#endpoints)
    - [Example API Test](#example-api-test)
- [Troubleshooting](#troubleshooting)

# Supported Environments:

| System             | CPU | GPU | Notes                                                                                                 |
| ------------------ | --- | --- | ------------------------------------------------------------------------------------------------------ |
| Windows 64-bit     | ✔   | ?   | Some Windows versions require the [VC runtime library](https://www.ghxi.com/yxkhj.html).               |
| Windows 32-bit     | ✔   | ?   | Static linking is unsupported; some versions need the [VC runtime library](https://www.ghxi.com/yxkhj.html). |
| Linux 64 / ARM64   | ✔   | ?   | May require upgrading the glibc version; see [glibc upgrade guide](https://www.cnblogs.com/fireinstone/p/18169273). |
| Linux 32-bit       | ✘   | ?   |                                                                                                        |
| macOS X64          | ✔   | ?   | For M1/M2/M3 chips, refer to [issue #67](https://github.com/sml2h3/ddddocr/issues/67).                 |

# Installation Steps

## From Source
- The `lib.rs` file implements `ddddocr`.
- The `main.rs` file implements `ocr_api_server`.
- The `model` directory contains models and character sets.
- To include this library in your project, add: `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master" }`
- To enable CUDA support: `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["cuda"] }`

- The project supports both static and dynamic linking. By default, it uses static linking and will automatically download the necessary libraries during the build process. Ensure your proxy settings are configured correctly. Note that the CUDA feature does not support static linking and will download dynamic libraries as needed.

## Precompiled Binaries
- If you prefer not to build from source, precompiled binaries are available in the [releases section](https://github.com/86maid/ddddocr/releases).

## GitHub Actions
- You can also use the configured [GitHub Actions](https://github.com/86maid/ddddocr/tree/master/.github/workflows) for building the project.

# Usage Documentation

## OCR Recognition
### Content Recognition
- Designed to recognize single-line text, such as common alphanumeric CAPTCHAs. The project supports Chinese, English (with options for case sensitivity), numbers, and certain special characters.
- Example:
  ```rust
  let image = std::fs::read("target.png").unwrap();
  let mut ocr = ddddocr::ddddocr_classification().unwrap();
  let res = ocr.classification(image, false).unwrap();
  println!("{:?}", res);
  ```
### Legacy Model
- To use the previous model:
  ```rust
  let image = std::fs::read("target.png").unwrap();
  let mut ocr = ddddocr::ddddocr_classification_old().unwrap();
  let res = ocr.classification(image, false).unwrap();
  println!("{:?}", res);
  ```
### Transparent Black PNG Support
- For images in transparent black PNG format, use the `png_fix` parameter:
  ```rust
  classification(image, true);
  ```
### Reference Images
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

## Object Detection
- Example:
  ```rust
  let image = std::fs::read("target.png").unwrap();
  let mut det = ddddocr::ddddocr_detection().unwrap();
  let res = det.detection(image).unwrap();
  println!("{:?}", res);
  ```
### Reference Images
![Test](https://cdn.wenanzhe.com/img/page1_1.jpg)
![Test](https://cdn.wenanzhe.com/img/page1_2.jpg)
![Test](https://cdn.wenanzhe.com/img/page1_3.jpg)
![Test](https://cdn.wenanzhe.com/img/page1_4.jpg)
![Test](https://cdn.wenanzhe.com/img/result.jpg)
![Test](https://cdn.wenanzhe.com/img/result2.jpg)
![Test](https://cdn.wenanzhe.com/img/result4.jpg)
Sample images are available to illustrate object detection capabilities.

## Slider Matching
### Algorithm 1
The small slider is a separate PNG image with a transparent background, as shown below:

![Test](https://cdn.wenanzhe.com/img/b.png)

Then, the background contains the slot for the small slider, as shown below:

![Test](https://cdn.wenanzhe.com/img/a.png)

- Example:
  ```rust
  let target_bytes = std::fs::read("target.png").unwrap();
  let background_bytes = std::fs::read("background.png").unwrap();
  let res = ddddocr::slide_match(target_bytes, background_bytes).unwrap();
  println!("{:?}", res);
  ```
### Algorithm 2
One image contains the slot (as shown below):

![Test](https://cdn.wenanzhe.com/img/bg.jpg)

Another image is the original (as shown below):

![Test](https://cdn.wenanzhe.com/img/fullpage.jpg)

- Example:
  ```rust
  let target_bytes = std::fs::read("target.png").unwrap();
  let background_bytes = std::fs::read("background.png").unwrap();
  let res = ddddocr::slide_comparison(target_bytes, background_bytes).unwrap();
  println!("{:?}", res);
  ```

## OCR Probability Output
- To provide more flexible control over OCR results, the project supports setting character ranges.
  - **Character Range Parameters:**
  
    | Parameter Value | Description                                      |
    |-----------------|------------------------------------------------|
    | 0               | Digits 0-9                                      |
    | 1               | Lowercase letters a-z                           |
    | 2               | Uppercase letters A-Z                           |
    | 3               | Lowercase and uppercase letters a-z, A-Z        |
    | 4               | Lowercase letters a-z and digits 0-9            |
    | 5               | Uppercase letters A-Z and digits 0-9            |
    | 6               | Lowercase and uppercase letters a-z, A-Z, and digits 0-9 |
    | 7               | Default character set: lowercase a-z, uppercase A-Z, and digits 0-9 |

For custom character sets, provide a string without spaces, where each character represents a candidate, e.g., `"0123456789+-x/="`.

Example usage:

```rust
let image = std::fs::read("image.png").unwrap();
let mut ocr = ddddocr::ddddocr_classification().unwrap();

// The number 3 corresponds to the enum CharsetRange::LowercaseUppercase;
// there's no need to specify the enum explicitly.
// ocr.set_ranges(3);

// Custom character set
ocr.set_ranges("0123456789+-x/=");

let result = ocr.classification_probability(image, false).unwrap();

// Note: The output might be extensive; be cautious to avoid performance issues.
println!("Probabilities: {}", result.json());

println!("Recognition result: {}", result.get_text());

```

## Custom OCR Model Import

The project supports importing custom models trained using [dddd_trainer](https://github.com/sml2h3/dddd_trainer).

Example usage:

```rust
use ddddocr::*;

let mut ocr = Ddddocr::with_model_charset(
    "myproject_0.984375_139_13000_2022-02-26-15-34-13.onnx",
    "charsets.json",
).unwrap();
let image_bytes = std::fs::read("888e28774f815b01e871d474e5c84ff2.jpg").unwrap();
let res = ocr.classification(&image_bytes).unwrap();
println!("{:?}", res);
```

# OCR API Server Example

The `ocr_api_server` provides a simple API server for OCR tasks.

## Usage
```cmd
Usage: ddddocr.exe [OPTIONS]

Options:
  -a, --address <ADDRESS>
            Listening address [default: 127.0.0.1]
  -p, --port <PORT>                
            Listening port [default: 9898]
  -f, --full                       
            Enable all options
      --jsonp                      
            Enable cross-origin requests; requires a query parameter specifying the callback function name; cannot use file (multipart) to pass parameters, e.g., http://127.0.0.1:9898/ocr/b64/text?callback=handle&image=xxx
      --ocr                        
            Enable content recognition; supports both new and old models
      --old                        
            Enable old model content recognition; supports both new and old models
      --det                        
            Enable object detection
      --ocr-probability <OCR_PROBABILITY>
            Enable content probability recognition; supports both new and old models; can only use official models; if the parameter is 0 to 7, it corresponds to the built-in character sets; if the parameter is an empty string, it indicates the default character set; other parameters indicate custom character sets, e.g., "0123456789+-x/="
      --old-probability <OLD_PROBABILITY>
            Enable old model content probability recognition; supports both new and old models; can only use official models; if the parameter is 0 to 7, it corresponds to the built-in character sets; if the parameter is an empty string, it indicates the default character set; other parameters indicate custom character sets, e.g., "0123456789+-x/="
      --ocr-path <OCR_PATH>        
            Path to content recognition model and character set; uses hash value to determine if it's a custom model; using a custom model will disable the old option; path model/common corresponds to model/common.onnx and character set model/common.json [default: model/common]
      --det-path <DET_PATH>        
            Path to object detection model [default: model/common_det.onnx]
      --slide-match                
            Enable slider recognition
      --simple-slide-match         
            Enable simple slider recognition
      --slide-compare              
            Enable slot recognition
  -h, --help                       
            Print help
```

## Endpoints

To test if the server is running, send a `GET` or `POST` request to `http://{host}:{port}/ping`. A successful response will return `pong`.

```
http://{host}:{port}/{opt}/{img_type}/{ret_type}

opt:
  ocr               Content recognition
  old               Old model content recognition
  det               Object detection
  ocr_probability   Content probability recognition
  old_probability   Old model content probability recognition
  match             Slider matching
  simple_match      Simple slider matching
  compare           Slot matching

img_type:
  file          File, i.e., multipart/form-data
  b64           Base64, i.e., {"a": encode(bytes), "b": encode(bytes)}

ret_type:
  json          JSON; success: {"status": 200, "result": object}, failure: {"status": 404, "msg": "failure reason"}
  text          Text; failure returns an empty string
```

## Example API Test

```python
import requests
import base64

host = "http://127.0.0.1:9898"
file = open('./image/3.png', 'rb').read()

# Test JSONP; can only use b64, not file
api_url = f"{host}/ocr/b64/text"
resp = requests.get(api_url, params={
  "callback": "handle",
  "image": base64.b64encode(file).decode(),
})
print(f"jsonp, api_url={api_url}, resp.text={resp.text}")

# Test OCR
api_url = f"{host}/ocr/file/text"
resp = requests.post(api_url, files={'image': file})
print(f"api_url={api_url}, resp.text={resp.text}")
```

# Troubleshooting

- Ensure both CUDA and cuDNN are properly installed.

- For CUDA 12, cuDNN 9.x is required.

- For CUDA 11, cuDNN 8.x is required.

- It's uncertain whether CUDA 10 is supported.

- By default, static linking is used, and necessary libraries are automatically downloaded during the build process. Ensure your proxy settings are configured correctly. The CUDA feature does not support static linking and will download dynamic libraries as needed.

- To specify the path for static linking libraries, set the `ORT_LIB_LOCATION` environment variable. Once set, automatic downloading of libraries will be disabled.

- For example, if the library path is `onnxruntime\build\Windows\Release\Release\onnxruntime.lib`, set `ORT_LIB_LOCATION` to `onnxruntime\build\Windows\Release`. 

- **Static Linking:** By default, the project uses static linking and will automatically download the necessary libraries during the build process. Ensure your proxy settings are correctly configured. Note that the `cuda` feature does not support static linking and will download dynamic libraries automatically.

- **Specifying Library Paths:** To specify the path for static libraries, set the `ORT_LIB_LOCATION` environment variable. Once set, the build process will not automatically download the libraries. For example, if your library path is `onnxruntime\build\Windows\Release\Release\onnxruntime.lib`, set `ORT_LIB_LOCATION` to `onnxruntime\build\Windows\Release`.

- **Automatic Library Downloads:** The `download-binaries` feature is enabled by default, which automatically downloads the necessary libraries. These libraries are stored in `C:\Users\<YourUsername>\AppData\ort.pyke.io`.

- **Dynamic Linking:** To enable dynamic linking, use the following configuration: `ddddocr = { git = "https://github.com/86maid/ddddocr.git", branch = "master", features = ["load-dynamic"] }`

-  After enabling the `load-dynamic` feature, you can specify the path to the `onnxruntime` dynamic library using `Ddddocr::set_onnxruntime_path`.

- **Manual Library Management:** With the `load-dynamic` feature enabled, the build process will not automatically download the `onnxruntime` library. You must manually download the `onnxruntime` library and place it in the program's runtime directory (or system API directory). This eliminates the need to call `Ddddocr::set_onnxruntime_path` again.

- **Windows Static Linking Issues:** If you encounter static linking failures on Windows, consider installing Visual Studio 2022.

- **Linux x86-64 Static Linking Issues:** For static linking failures on Linux x86-64, install `gcc11` and `g++11`. Ensure your Ubuntu version is 20.04 or higher.

- **Linux ARM64 Static Linking Issues:** On Linux ARM64, static linking failures may require `glibc` version 2.35 or higher (Ubuntu 22.04 or above).

- **macOS Static Linking Issues:** For macOS, static linking requires macOS version 10.15 or higher.

- **CUDA Testing Issues:** When running `cargo test` with CUDA enabled, you might encounter a panic with exit code `0xc000007b`. This occurs because the automatically generated dynamic library is located in the `target/debug` directory. Manually copy it to the `target/debug/deps` directory, as CUDA currently does not support static linking.

- **Dynamic Linking Requirements:** Dynamic linking requires [onnxruntime](https://github.com/microsoft/onnxruntime/releases/tag/v1.18.1) version 1.18.x.

For more detailed troubleshooting and information, visit [ort.pyke.io](https://ort.pyke.io/). 
