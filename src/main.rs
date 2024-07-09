use actix_multipart::Multipart;
use actix_web::{
    route,
    test::TestRequest,
    web::{self, Bytes, BytesMut, Payload, Query},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use anyhow::ensure;
use base64::{engine::general_purpose, Engine};
use clap::Parser;
use ddddocr::{
    ddddocr_classification, ddddocr_classification_old, ddddocr_detection, Charset, Ddddocr,
    MapJson,
};
use futures_util::StreamExt;
use std::{collections::HashMap, fmt::Debug, str::FromStr};

static mut OCR: Option<Ddddocr<'static>> = None;
static mut OLD: Option<Ddddocr<'static>> = None;
static mut OCR_PROBABILITY: Option<Ddddocr<'static>> = None;
static mut OLD_PROBABILITY: Option<Ddddocr<'static>> = None;
static mut DET: Option<Ddddocr<'static>> = None;
static mut FLAG: u32 = 0;

#[derive(Parser, Debug)]
struct MainArgs {
    /// 监听地址
    #[arg(short, long, default_value_t = { "127.0.0.1".to_string() })]
    address: String,

    /// 监听端口
    #[arg(short, long, default_value_t = 9898)]
    port: u16,

    /// 开启所有选项
    #[arg(short, long)]
    full: bool,

    /// 开启跨域，需要一个 query 指定回调函数的名字，不能使用 file (multipart) 传递参数，
    /// 例如 http://127.0.0.1:9898/ocr/b64/text?callback=handle&image=xxx
    #[arg(long)]
    jsonp: bool,

    /// 开启内容识别，支持新旧模型共存
    #[arg(long)]
    ocr: bool,

    /// 开启旧版模型内容识别，支持新旧模型共存
    #[arg(long)]
    old: bool,

    /// 开启目标检测
    #[arg(long)]
    det: bool,

    /// 开启内容概率识别，支持新旧模型共存，只能使用官方模型，
    /// 如果参数是 0 到 7，对应内置的字符集，
    /// 如果参数为空字符串，表示默认字符集，
    /// 除此之外的参数，表示自定义字符集，例如 "0123456789+-x/="
    #[arg(long)]
    ocr_probability: Option<String>,

    /// 开启旧版模型内容概率识别，支持新旧模型共存，只能使用官方模型，
    /// 如果参数是 0 到 7，对应内置的字符集，
    /// 如果参数为空字符串，表示默认字符集，
    /// 除此之外的参数，表示自定义字符集，例如 "0123456789+-x/="
    #[arg(long)]
    old_probability: Option<String>,

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
                // 不要使用双引号，因为 value 是数组
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
            // 失败返回空文本
            "".to_string()
        }
    };

    let qs = request.query_string().to_string();

    let mut callback = String::new();

    if unsafe { FLAG } & 8 == 8 {
        if let Ok(v) = Query::<HashMap<String, String>>::from_query(&qs) {
            if let Some(v) = v.get("callback") {
                if v.is_empty() {
                    return HttpResponse::BadRequest()
                        .content_type("application/javascript")
                        .body("alert(\"预期之外的 query: 找到键名 callback, 但是没有键值\")");
                }

                callback = v.clone();
            }
        };
    }

    unsafe {
        let inner = || async {
            match option.as_str() {
                "ocr" if OCR.is_some() => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["image"],
                    )
                    .await?;

                    ensure!(file.len() == 1, "不支持多个图片");

                    let file = file[0].1.clone();

                    let ddddocr = OCR.as_mut().unwrap();

                    Ok(
                        tokio::task::spawn_blocking(move || ddddocr.classification(file, false))
                            .await
                            .unwrap()?,
                    )
                }
                "old" if OLD.is_some() => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["image"],
                    )
                    .await?;

                    ensure!(file.len() == 1, "不支持多个图片");

                    let file = file[0].1.clone();

                    let ddddocr = OLD.as_mut().unwrap();

                    Ok(
                        tokio::task::spawn_blocking(move || ddddocr.classification(file, false))
                            .await
                            .unwrap()?,
                    )
                }
                "det" if DET.is_some() => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["image"],
                    )
                    .await?;

                    ensure!(file.len() == 1, "不支持多个图片");

                    let file = file[0].1.clone();

                    let ddddocr = DET.as_mut().unwrap();

                    Ok(tokio::task::spawn_blocking(move || ddddocr.detection(file))
                        .await
                        .unwrap()?
                        .json())
                }
                "ocr_probability" if OCR_PROBABILITY.is_some() => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["image"],
                    )
                    .await?;

                    ensure!(file.len() == 1, "不支持多个图片");

                    let file = file[0].1.clone();

                    let ddddocr = OCR_PROBABILITY.as_mut().unwrap();

                    Ok(tokio::task::spawn_blocking(move || {
                        ddddocr
                            .classification_probability(file, false)
                            .map(|mut v| {
                                v.get_text();
                                v.json()
                            })
                    })
                    .await
                    .unwrap()?)
                }
                "old_probability" if OLD_PROBABILITY.is_some() => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["image"],
                    )
                    .await?;

                    ensure!(file.len() == 1, "不支持多个图片");

                    let file = file[0].1.clone();

                    let ddddocr = OLD_PROBABILITY.as_mut().unwrap();

                    Ok(tokio::task::spawn_blocking(move || {
                        ddddocr
                            .classification_probability(file, false)
                            .map(|mut v| {
                                v.get_text();
                                v.json()
                            })
                    })
                    .await
                    .unwrap()?)
                }
                "match" if FLAG & 1 == 1 => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["target", "background"],
                    )
                    .await?;

                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "预期两张图片, 但是找到两个相同的 key {}",
                        file[0].0
                    );

                    if file[0].0 == "target" {
                        ddddocr::slide_match(file[0].1.clone(), file[1].1.clone()).map(|v| v.json())
                    } else {
                        ddddocr::slide_match(file[1].1.clone(), file[0].1.clone()).map(|v| v.json())
                    }
                }
                "simple_match" if FLAG & 2 == 2 => {
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["target", "background"],
                    )
                    .await?;

                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "预期两张图片, 但是找到两个相同的 key {}",
                        file[0].0
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
                    let file = get_file_jsonp(
                        image_type,
                        content,
                        request,
                        !callback.is_empty(),
                        &["target", "background"],
                    )
                    .await?;

                    ensure!(
                        file.len() == 2
                            && (file[0].0 == "target" && file[1].0 == "background"
                                || file[0].0 == "background" && file[1].0 == "target"),
                        "预期两张图片, 但是找到两个相同的 key {}",
                        file[0].0
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

        let result = inner()
            .await
            .map(map_ok)
            .unwrap_or_else(|v| map_error(v.to_string()));

        // jsonp
        if !callback.is_empty() {
            return HttpResponse::Ok()
                .content_type("application/javascript")
                .body(format!(
                    "{}(\"{}\")",
                    callback,
                    result.replace("\"", "\\\"")
                ));
        }

        HttpResponse::Ok().body(result)
    }
}

async fn get_file_jsonp(
    image_type: String,
    content: Payload,
    request: HttpRequest,
    jsonp: bool,
    keys: &[&str],
) -> anyhow::Result<Vec<(String, Bytes)>> {
    if jsonp {
        // 把 query 转换成 request，这样就可以偷懒啦！
        let data = serde_json::to_string(
            &Query::<HashMap<String, String>>::from_query(request.query_string())
                .unwrap()
                .0,
        )
        .map_err(|v| anyhow::anyhow!("预期之外的 query: {}", v))?;

        let (a, mut b) = TestRequest::default().set_payload(data).to_http_parts();

        let pl = <Payload as actix_web::FromRequest>::from_request(&a, &mut b)
            .await
            .unwrap();

        // 这里的 request 不适用于 jsonp，只有 multipart 才有效
        get_file(image_type, pl, request, keys).await
    } else {
        get_file(image_type, content, request, keys).await
    }
}

async fn get_file(
    image_type: String,
    mut content: Payload,
    request: HttpRequest,
    keys: &[&str],
) -> anyhow::Result<Vec<(String, Bytes)>> {
    fn get_type_name(value: &serde_json::Value) -> &'static str {
        if value.is_string() {
            "string"
        } else if value.is_number() {
            "number"
        } else if value.is_boolean() {
            "boolean"
        } else if value.is_array() {
            "array"
        } else if value.is_object() {
            "object"
        } else {
            "unknown"
        }
    }

    ensure!(
        image_type == "b64" || image_type == "file",
        "预期 b64 or file, 但是找到 {}",
        image_type
    );

    let mut result = Vec::new();

    if image_type == "b64" {
        let mut buffer = BytesMut::new();

        while let Some(v) = content.next().await {
            buffer.extend(v.map_err(|v| anyhow::anyhow!(v))?);
        }

        let buffer = serde_json::from_slice::<serde_json::Value>(&buffer)
            .map_err(|v| anyhow::anyhow!("无法解析 json: {}", v))?;

        let buffer = buffer.as_object().ok_or(anyhow::anyhow!(
            "预期之外的类型: 预期 {{ \"image\": base64, ... }}, 但是找到 {}",
            get_type_name(&buffer)
        ))?;

        for i in buffer {
            let key = i.0.to_string();

            if keys.contains(&key.as_str()) {
                let value = Bytes::from(
                    general_purpose::STANDARD
                        .decode(
                            i.1.as_str()
                                .map(|v| {
                                    // 删除 base64 的文件头
                                    if let Some(pos) = v.find(',') {
                                        &v[pos + 1..]
                                    } else {
                                        v
                                    }
                                })
                                .ok_or(anyhow::anyhow!(
                            "预期之外的类型: key {} 对应的 value 预期的类型是 string, 但是找到 {}",
                            i.0,
                            get_type_name(i.1)
                        ))?,
                        )
                        .map_err(|v| {
                            anyhow::anyhow!(
                                "无法解码 base64: key {} 对应的 value 解码失败: {}",
                                i.0,
                                v
                            )
                        })?,
                );

                result.push((key, value));
            }
        }
    } else {
        let mut stream = Multipart::new(request.headers(), content);

        while let Some(v) = stream.next().await {
            let mut v = v.map_err(|v| anyhow::anyhow!("{v}"))?;

            if keys.contains(&v.name()) {
                let mut buffer = BytesMut::new();

                while let Some(v) = v.next().await {
                    buffer.extend(v.map_err(|v| anyhow::anyhow!("{v}"))?);
                }

                result.push((v.name().to_string(), buffer.freeze()));
            }
        }
    }

    if result.is_empty() {
        anyhow::bail!("预期 json or multipart, 但是找到空数据");
    }

    Ok(result)
}

#[actix_web::main]
async fn main() {
    let args = MainArgs::parse();
    let mut diy = false;

    unsafe {
        if args.jsonp {
            FLAG |= 8;

            println!("开启 jsonp 成功");
        }

        if cfg!(feature = "inline-model") {
            if args.ocr || args.full {
                OCR = Some(ddddocr_classification().expect("开启内容识别失败"));

                println!("开启内容识别成功");
            }

            if args.old || args.full && !diy {
                OLD = Some(ddddocr_classification_old().expect("开启旧版模型内容识别失败"));

                println!("开启旧版模型内容识别成功");
            }

            if args.det || args.full {
                DET = Some(ddddocr_detection().expect("开启目标检测失败"));

                println!("开启目标检测成功");
            }

            if args.ocr_probability.is_some() || args.full {
                OCR_PROBABILITY = Some({
                    let mut result = ddddocr_classification().expect("开启内容概率识别失败");

                    let text = args.ocr_probability.clone().unwrap_or(String::new());

                    match text.chars().next() {
                        Some(v) => match v {
                            '0'..='7' if text.len() == 1 => {
                                result.set_ranges(v as i32 - '0' as i32)
                            }
                            _ => result.set_ranges(text.as_str()),
                        },
                        None => {}
                    }

                    result
                });

                println!("开启内容概率识别成功");
            }

            if args.old_probability.is_some() || args.full {
                OLD_PROBABILITY = Some({
                    let mut result =
                        ddddocr_classification_old().expect("开启旧版模型内容概率识别失败");

                    let text = args.old_probability.clone().unwrap_or(String::new());

                    match text.chars().next() {
                        Some(v) => match v {
                            '0'..='7' if text.len() == 1 => {
                                result.set_ranges(v as i32 - '0' as i32)
                            }
                            _ => result.set_ranges(text.as_str()),
                        },
                        None => {}
                    }

                    result
                });

                println!("开启旧版模型内容概率识别成功");
            }
        } else {
            if args.ocr || args.full {
                let model = std::fs::read(args.ocr_path.clone() + ".onnx").expect("打开模型失败");

                let charset = std::fs::read_to_string(args.ocr_path.clone() + ".json")
                    .expect("打开字符集失败");

                diy = ddddocr::is_diy(&model);

                OCR = Some(
                    Ddddocr::new(&model, Charset::from_str(&charset).expect("解析字符集失败"))
                        .expect("开启内容识别失败"),
                );

                println!("开启内容识别成功");
            }

            if args.old || args.full && !diy {
                let model =
                    std::fs::read(args.ocr_path.clone() + "_old.onnx").expect("打开模型失败");

                let charset = std::fs::read_to_string(args.ocr_path.clone() + "_old.json")
                    .expect("打开字符集失败");

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

            if args.ocr_probability.is_some() || args.full {
                let model = std::fs::read(args.ocr_path.clone() + ".onnx").expect("打开模型失败");

                let charset = std::fs::read_to_string(args.ocr_path.clone() + ".json")
                    .expect("打开字符集失败");

                if ddddocr::is_diy(&model) {
                    panic!("内容概率识别只能使用官方模型");
                }

                OCR_PROBABILITY = Some({
                    let mut result =
                        Ddddocr::new(&model, Charset::from_str(&charset).expect("解析字符集失败"))
                            .expect("开启内容概率识别失败");

                    let text = args.ocr_probability.clone().unwrap_or(String::new());

                    match text.chars().next() {
                        Some(v) => match v {
                            '0'..='7' if text.len() == 1 => {
                                result.set_ranges(v as i32 - '0' as i32)
                            }
                            _ => result.set_ranges(text.as_str()),
                        },
                        None => {}
                    }

                    result
                });

                println!("开启内容概率识别成功");
            }

            if args.old_probability.is_some() || args.full {
                let model =
                    std::fs::read(args.ocr_path.clone() + "_old.onnx").expect("打开模型失败");

                let charset =
                    std::fs::read_to_string(args.ocr_path + "_old.json").expect("打开字符集失败");

                if ddddocr::is_diy(&model) {
                    panic!("内容概率识别只能使用官方模型");
                }

                OLD_PROBABILITY = Some({
                    let mut result =
                        Ddddocr::new(&model, Charset::from_str(&charset).expect("解析字符集失败"))
                            .expect("开启旧版模型内容概率识别失败");

                    let text = args.old_probability.clone().unwrap_or(String::new());

                    match text.chars().next() {
                        Some(v) => match v {
                            '0'..='7' if text.len() == 1 => {
                                result.set_ranges(v as i32 - '0' as i32)
                            }
                            _ => result.set_ranges(text.as_str()),
                        },
                        None => {}
                    }

                    result
                });

                println!("开启旧版模型内容概率识别成功");
            }
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
        || args.ocr_probability.is_some()
        || args.old_probability.is_some()
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
        println!("没有开启任何服务，请使用 --help 查看帮助");
    }
}
