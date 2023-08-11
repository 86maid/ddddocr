use actix_multipart::Multipart;
use actix_web::{
    route,
    web::{self},
    web::{Bytes, BytesMut, Payload},
    App, HttpRequest, HttpServer, Responder,
};
use anyhow::ensure;
use base64::{engine::general_purpose, Engine};
use clap::Parser;
use ddddocr::{Charset, Ddddocr, MapJson};
use futures_util::StreamExt;
use std::{fmt::Debug, str::FromStr};

static mut OCR: Option<Ddddocr<'static>> = None;
static mut OLD: Option<Ddddocr<'static>> = None;
static mut DET: Option<Ddddocr<'static>> = None;
static mut FLAG: u32 = 0;

#[derive(Parser, Debug)]
struct Args {
    /// 监听地址
    #[arg(short, long, default_value_t = { "127.0.0.1".to_string() })]
    address: String,

    /// 监听端口
    #[arg(short, long, default_value_t = 9898)]
    port: u16,

    /// 开启所有选项
    #[arg(short, long)]
    full: bool,

    /// 开启内容识别，支持新旧模型共存
    #[arg(long)]
    ocr: bool,

    /// 开启旧版模型内容识别，支持新旧模型共存
    #[arg(long)]
    old: bool,

    /// 开启目标检测
    #[arg(long)]
    det: bool,

    /// 内容识别模型以及字符集路径，
    /// 通过哈希值判断是否为自定义模型，
    /// 使用自定义模型会使 old 选项失效，
    /// 路径 model/common 对应模型 model/common.onnx 和字符集 model/common.json
    #[arg(long, default_value_t = { "model/common".to_string() })]
    ocr_path: String,

    /// 目标检测模型路径
    #[arg(long, default_value_t = { "model/common_det.onnx".to_string() })]
    det_path: String,

    /// 开启滑块识别
    #[arg(long)]
    slide_match: bool,

    /// 开启简单滑块识别
    #[arg(long)]
    simple_slide_match: bool,

    /// 开启坑位识别
    #[arg(long)]
    slide_compare: bool,
}

#[route("/ping", method = "GET", method = "POST")]
async fn ping() -> impl Responder {
    "pong"
}

#[route("/{a}/{b}/{c}", method = "GET", method = "POST")]
async fn handle_abc(
    args: web::Path<(String, String, String)>,
    content: Payload,
    request: HttpRequest,
) -> impl Responder {
    let (option, image_type, result_type) = args.into_inner();
    let map_ok = |value: String| {
        if result_type == "json" {
            if option == "ocr" || option == "old" {
                serde_json::json!({
                    "status": 200,
                    "result": value,
                })
                .to_string()
            } else {
                // 不要双引号
                format!(r#"{{"status":200,"result":{}}}"#, value)
            }
        } else {
            value
        }
    };
    let map_error = |value: String| {
        if result_type == "json" {
            serde_json::json!({
                "status": 404,
                "msg": value,
            })
            .to_string()
        } else {
            "".to_string()
        }
    };
    unsafe {
        let inner = || async {
            match option.as_str() {
                "ocr" if OCR.is_some() => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(file.iter().any(|v| v.0 == "image") && file.len() == 1);
                    let file = file[0].1.clone();
                    let ddddocr = OCR.as_mut().unwrap();
                    Ok(
                        tokio::task::spawn_blocking(move || ddddocr.classification(file))
                            .await
                            .unwrap()?,
                    )
                }
                "old" if OLD.is_some() => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(file.iter().any(|v| v.0 == "image") && file.len() == 1);
                    let file = file[0].1.clone();
                    let ddddocr = OLD.as_mut().unwrap();
                    Ok(
                        tokio::task::spawn_blocking(move || ddddocr.classification(file))
                            .await
                            .unwrap()?,
                    )
                }
                "det" if DET.is_some() => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(
                        file.len() == 1 && file[0].0 == "image",
                        "找不到名为 image 的文件"
                    );
                    let file = file[0].1.clone();
                    let ddddocr = DET.as_mut().unwrap();
                    Ok(tokio::task::spawn_blocking(move || ddddocr.detection(file))
                        .await
                        .unwrap()?
                        .json())
                }
                "match" if FLAG & 1 == 1 => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "找不到名为 target 或 background 的文件"
                    );
                    if file[0].0 == "target" {
                        ddddocr::slide_match(file[0].1.clone(), file[1].1.clone()).map(|v| v.json())
                    } else {
                        ddddocr::slide_match(file[1].1.clone(), file[0].1.clone()).map(|v| v.json())
                    }
                }
                "simple_match" if FLAG & 2 == 2 => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "找不到名为 target 或 background 的文件"
                    );
                    if file[0].0 == "target" {
                        ddddocr::simple_slide_match(file[0].1.clone(), file[1].1.clone())
                            .map(|v| v.json())
                    } else {
                        ddddocr::simple_slide_match(file[1].1.clone(), file[0].1.clone())
                            .map(|v| v.json())
                    }
                }
                "compare" if FLAG & 4 == 4 => {
                    let file = get_file(image_type, content, request).await?;
                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "找不到名为 target 或 background 的文件"
                    );
                    if file[0].0 == "target" {
                        ddddocr::slide_comparison(file[0].1.clone(), file[1].1.clone())
                            .map(|v| v.json())
                    } else {
                        ddddocr::slide_comparison(file[1].1.clone(), file[0].1.clone())
                            .map(|v| v.json())
                    }
                }
                _ => Err(anyhow::anyhow!(
                    "预期之外的选项: {option}, 请确认服务是否已开启"
                )),
            }
        };
        inner()
            .await
            .map(map_ok)
            .unwrap_or_else(|v| map_error(v.to_string()))
    }
}

async fn get_file(
    image_type: String,
    mut content: Payload,
    request: HttpRequest,
) -> anyhow::Result<Vec<(String, Bytes)>> {
    ensure!(
        image_type == "b64" || image_type == "file",
        "预期 b64, file 找到 {}",
        image_type
    );
    let mut result = Vec::new();
    if image_type == "b64" {
        let mut buffer = BytesMut::new();
        while let Some(v) = content.next().await {
            buffer.extend(v.map_err(|v| anyhow::anyhow!(v))?);
        }
        let buffer = serde_json::from_slice::<serde_json::Value>(&buffer)?;
        let buffer = buffer
            .as_object()
            .ok_or(anyhow::anyhow!("无法解析 base64 解码后的 json 文本"))?;
        for i in buffer {
            let temp = (
                i.0.to_string(),
                Bytes::from(
                    general_purpose::STANDARD.decode(
                        i.1.as_str()
                            .ok_or(anyhow::anyhow!("无法解码 json 中的 base64 文本: {}", i.0))?,
                    )?,
                ),
            );
            result.push(temp);
        }
    } else {
        let mut stream = Multipart::new(request.headers(), content);
        while let Some(v) = stream.next().await {
            let mut v = v.map_err(|v| anyhow::anyhow!("{v}"))?;
            let mut buffer = BytesMut::new();
            while let Some(v) = v.next().await {
                buffer.extend(v.map_err(|v| anyhow::anyhow!("{v}"))?);
            }
            result.push((v.name().to_string(), buffer.freeze()));
        }
    }
    if result.is_empty() {
        return Err(anyhow::anyhow!("预期 json, multipart, 找到空数据"));
    }
    Ok(result)
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();
    let mut diy = false;

    unsafe {
        if args.ocr || args.full {
            let model = std::fs::read(args.ocr_path.clone() + ".onnx").expect("打开模型失败");
            let charset =
                std::fs::read_to_string(args.ocr_path.clone() + ".json").expect("打开字符集失败");
            diy = ddddocr::is_diy(&model);
            OCR = Some(
                Ddddocr::new(&model, Charset::from_str(&charset).expect("解析字符集失败"))
                    .expect("开启内容识别失败"),
            );
            println!("开启内容识别成功");
        }

        if args.old || args.full && !diy {
            let model = std::fs::read(args.ocr_path.clone() + "_old.onnx").expect("打开模型失败");
            let charset =
                std::fs::read_to_string(args.ocr_path + "_old.json").expect("打开字符集失败");
            OLD = Some(
                Ddddocr::new(&model, Charset::from_str(&charset).expect("解析字符集失败"))
                    .expect("开启旧版模型内容识别失败"),
            );
            println!("开启旧版模型内容识别成功");
        }

        if args.det || args.full {
            let model = std::fs::read(&args.det_path).expect("打开模型失败");
            DET = Some(Ddddocr::new_model(&model).expect("开启目标检测失败"));
            println!("开启目标检测成功");
        }

        if args.slide_match || args.full {
            FLAG |= 1;
            println!("开启滑块识别成功");
        }

        if args.simple_slide_match || args.full {
            FLAG |= 2;
            println!("开启简单滑块识别成功");
        }

        if args.slide_compare || args.full {
            FLAG |= 4;
            println!("开启坑位识别成功");
        }
    }

    if args.full
        || args.ocr
        || args.old
        || args.det
        || args.simple_slide_match
        || args.slide_match
        || args.slide_compare
    {
        println!("正在监听 {}:{}", args.address, args.port);
        HttpServer::new(|| App::new().service(ping).service(handle_abc))
            .bind((args.address, args.port))
            .expect("地址绑定失败")
            .run()
            .await
            .expect("监听失败");
    } else {
        println!("没有开启任何服务");
    }
}
