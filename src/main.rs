use base64::prelude::*;
use clap::ArgGroup;
use clap::Parser;
use ddddocr::*;
use enable_ansi_support::enable_ansi_support;
use lru::LruCache;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::model::ErrorCode;
use rmcp::model::JsonRpcError;
use rmcp::model::JsonRpcRequest;
use rmcp::model::JsonRpcResponse;
use rmcp::model::JsonRpcVersion2_0;
use rmcp::model::NumberOrString;
use rmcp::ErrorData;
use salvo::catcher::Catcher;
use salvo::http::request;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::num::NonZero;
use std::sync::LazyLock;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tracing::info;
use tracing::warn;

static ARGS: OnceLock<Args> = OnceLock::new();
static OCR: OnceLock<Ddddocr> = OnceLock::new();
static DET: OnceLock<Ddddocr> = OnceLock::new();
static CACHE: LazyLock<Mutex<LruCache<String, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZero::new(64).unwrap())));

#[derive(Parser, Debug, Clone)]
#[clap(
    group(
        ArgGroup::new("ocr_old")
            .args(&["ocr", "old"])
            .multiple(false)
    ),
    group(
        ArgGroup::new("mcp_only_mcp")
            .args(&["mcp", "only_mcp"])
            .multiple(false)
))]
struct Args {
    /// 监听地址。
    #[arg(long, default_value_t = { "0.0.0.0:8000".to_string() })]
    address: String,

    /// mcp 协议支持，与 only_mcp 互斥。
    #[arg(long)]
    mcp: bool,

    /// 仅开启 mcp 协议，不开启普通路由，与 mcp 互斥。
    #[arg(long)]
    only_mcp: bool,

    /// 开启内容识别，与 old 互斥。
    #[arg(long)]
    ocr: bool,

    /// 开启旧版模型内容识别，与 ocr 互斥。
    #[arg(long)]
    old: bool,

    /// 开启目标检测。
    #[arg(long)]
    det: bool,

    /// 开启滑块和坑位识别。
    #[arg(long)]
    slide: bool,

    /// 全局默认字符集，用于概率识别，  
    /// 如果 API 未提供字符集，则使用此参数，  
    /// 当值为 0~7 时，表示选择内置字符集，
    /// 其他值表示自定义字符集，例如 "0123456789+-x/="，  
    /// 如果未设置，则使用完整字符集，不做限制。
    #[arg(long)]
    ocr_charset_range: Option<String>,

    /// 内容识别模型以及字符集路径，
    /// 如果你开启了 features 的 inline-model 选项（默认开启），则不用管这个选项，除非你想使用自定义模型，
    /// 模型 model/common.onnx 和字符集 model/common.json 要同名。
    #[arg(long, default_value_t = { "model/common.onnx".to_string() })]
    ocr_path: String,

    /// 目标检测模型路径，
    /// 如果你开启了 features 的 inline-model 选项（默认开启），则不用管这个选项，除非你想使用自定义模型。
    #[arg(long, default_value_t = { "model/common_det.onnx".to_string() })]
    det_path: String,

    /// 输入你的域名，自动获取 SSL 证书，
    /// 即 https 的支持。
    #[arg(long)]
    acme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct OCRRequest {
    /// 要进行识别的图片，base64 编码。
    image: String,

    /// 如果 png_fix 为 true，则支持透明黑色背景的 png 图片。
    png_fix: Option<bool>,

    /// 是否返回概率信息。
    probability: Option<bool>,

    /// 限定字符范围，只对本次 ocr 生效，
    /// 如果参数是 0 到 7，对应内置的字符集，
    /// 除此之外的参数，表示自定义字符集，例如 `"0123456789+-x/="`。
    charset_range: Option<String>,

    /// 颜色过滤，例如 `red` 或 `["red", "blue"]` 或 `[[[0, 50, 50], [10, 255, 255]]]`。
    color_filter: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct OCRResponse {
    /// 识别结果。
    text: String,

    /// 概率信息。
    probability: Option<Vec<Vec<f32>>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct DETRequest {
    /// 要进行识别的图片，base64 编码。
    image: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct DETResponse {
    /// 包围盒坐标，例如 `[[x1, y1, x2, y2], [x1, y1, x2, y2]]`。
    bboxes: Vec<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SlideMatchRequest {
    /// 滑块图片，base64 编码。
    target_image: String,

    /// 背景图片，base64 编码。
    background_image: String,

    /// 是否为简单滑块。
    simple_target: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SlideMatchResponse {
    /// 目标位置坐标，例如 `[x1, y1, x2, y2]`。
    target: Vec<u32>,

    /// 透明部分的 x 偏移。
    target_x: u32,

    /// 透明部分的 y 偏移。
    target_y: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SlideComparisonRequest {
    /// 滑块图片，base64 编码。
    target_image: String,

    /// 背景图片，base64 编码。
    background_image: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SlideComparisonResponse {
    /// 坑位 x 偏移。
    x: u32,

    /// 坑位 y 偏移。
    y: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct StatusResponse {
    /// 服务状态。
    service_status: String,

    /// 已经开启的功能。
    enabled_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct APIResponse<T> {
    code: u16,
    msg: String,
    data: Option<T>,
}

#[endpoint(responses((status_code = 200, body = APIResponse<OCRResponse>)))]
async fn route_ocr(req: JsonBody<OCRRequest>, res: &mut Response) -> anyhow::Result<()> {
    let image = BASE64_STANDARD.decode(&req.image)?;
    let png_fix = req.png_fix.unwrap_or_default();
    let probability = req.probability.unwrap_or_default();

    let color_filter = if let Some(v) = req.color_filter.clone() {
        Some(serde_json::from_value::<ColorFilter>(v)?)
    } else {
        None
    };

    let charset_range = if let Some(ref v) = req.charset_range {
        let ocr_charset_range = match v.as_str() {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" => {
                CharsetRange::from(v.parse::<i32>().unwrap())
            }
            v => CharsetRange::from(v),
        };

        Some(CharsetRange::Charset(
            CACHE
                .lock()
                .await
                .get_or_insert(v.to_string(), || {
                    OCR.get().unwrap().calc_ranges(ocr_charset_range)
                })
                .clone(),
        ))
    } else {
        None
    };

    let (text, probability) = if charset_range.is_some() || probability {
        let mut result = spawn_blocking({
            let color_filter = color_filter.clone();
            let charset_range = charset_range.clone();

            move || {
                OCR.get().unwrap().classification_probability_with_options(
                    image,
                    png_fix,
                    color_filter,
                    charset_range,
                )
            }
        })
        .await??;

        (
            result.get_text().to_string(),
            probability.then_some(result.probability),
        )
    } else {
        let text = spawn_blocking({
            let color_filter = color_filter.clone();

            move || {
                OCR.get()
                    .unwrap()
                    .classification_with_options(image, png_fix, color_filter)
            }
        })
        .await??;

        (text, None)
    };

    let response = OCRResponse { text, probability };

    let response = APIResponse {
        code: 200,
        msg: "success".to_string(),
        data: Some(response),
    };

    res.render(Json(response));

    Ok(())
}

#[endpoint(responses((status_code = 200, body = APIResponse<DETResponse>)))]
async fn route_det(req: JsonBody<DETRequest>, res: &mut Response) -> anyhow::Result<()> {
    let image = BASE64_STANDARD.decode(&req.image)?;
    let bboxes = spawn_blocking(|| DET.get().unwrap().detection(image)).await??;
    let response = DETResponse {
        bboxes: bboxes.to_vec(),
    };

    let response = APIResponse {
        code: 200,
        msg: "success".to_string(),
        data: Some(response),
    };

    res.render(Json(response));

    Ok(())
}

#[endpoint(responses((status_code = 200, body = APIResponse<SlideMatchResponse>)))]
async fn route_slide_match(
    req: JsonBody<SlideMatchRequest>,
    res: &mut Response,
) -> anyhow::Result<()> {
    let target_image = BASE64_STANDARD.decode(&req.target_image)?;
    let background_image = BASE64_STANDARD.decode(&req.background_image)?;

    let result = spawn_blocking(move || {
        if req.simple_target.unwrap_or_default() {
            simple_slide_match(target_image, background_image)
        } else {
            slide_match(target_image, background_image)
        }
    })
    .await??;

    let response = SlideMatchResponse {
        target: vec![result.x1, result.y1, result.x2, result.y2],
        target_x: result.target_x,
        target_y: result.target_y,
    };

    let response = APIResponse {
        code: 200,
        msg: "success".to_string(),
        data: Some(response),
    };

    res.render(Json(response));

    Ok(())
}

#[endpoint(responses((status_code = 200, body = APIResponse<SlideComparisonResponse>)))]
async fn route_slide_comparison(
    req: JsonBody<SlideComparisonRequest>,
    res: &mut Response,
) -> anyhow::Result<()> {
    let target_image = BASE64_STANDARD.decode(&req.target_image)?;
    let background_image = BASE64_STANDARD.decode(&req.background_image)?;
    let result = spawn_blocking(|| slide_comparison(target_image, background_image)).await??;
    let response = SlideComparisonResponse {
        x: result.0,
        y: result.1,
    };

    let response = APIResponse {
        code: 200,
        msg: "success".to_string(),
        data: Some(response),
    };

    res.render(Json(response));

    Ok(())
}

#[endpoint(responses((status_code = 200, body = APIResponse<StatusResponse>)))]
async fn route_status(res: &mut Response) {
    let args = ARGS.get().unwrap();
    let mut enabled_features = Vec::new();

    if args.ocr || args.old {
        enabled_features.push("ocr".to_string());
    }

    if args.det {
        enabled_features.push("det".to_string());
    }

    if args.slide {
        enabled_features.push("slide".to_string());
    };

    if args.mcp {
        enabled_features.push("mcp".to_string());
    };

    if args.only_mcp {
        enabled_features.push("only_mcp".to_string());
    };

    let response = StatusResponse {
        service_status: "running".to_string(),
        enabled_features,
    };

    let response = APIResponse {
        code: 200,
        msg: "success".to_string(),
        data: Some(response),
    };

    res.render(Json(response));
}

#[handler]
async fn route_mcp(
    req_body: JsonBody<JsonRpcRequest<rmcp::model::ClientRequest>>,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) -> anyhow::Result<()> {
    match &req_body.request {
        rmcp::model::ClientRequest::InitializeRequest(_) => {
            res.render(Text::Json(include_str!("../initialize.json")))
        }
        rmcp::model::ClientRequest::ListToolsRequest(_) => {
            res.render(Text::Json(include_str!("../list.json")))
        }
        rmcp::model::ClientRequest::CallToolRequest(v) => {
            match v.params.name.as_ref() {
                "ocr" | "det" | "slide-match" | "slide-comparison" => {
                    let mut req = salvo::Request::new();

                    req.add_header("content-type", "application/json", true)?;

                    req.replace_body(ReqBody::Once(bytes::Bytes::from(
                        serde_json::to_value(v.params.arguments.clone())?.to_string(),
                    )));

                    let req = &mut req;

                    let args = ARGS.get().unwrap();

                    match v.params.name.as_ref() {
                        "ocr" if args.ocr => route_ocr.handle(req, depot, res, ctrl).await,
                        "det" if args.det => route_det.handle(req, depot, res, ctrl).await,
                        "slide-match" if args.slide => {
                            route_slide_match.handle(req, depot, res, ctrl).await
                        }
                        "slide-comparison" if args.slide => {
                            route_slide_comparison.handle(req, depot, res, ctrl).await
                        }
                        v => {
                            res.render(Json(JsonRpcError {
                                jsonrpc: JsonRpcVersion2_0,
                                id: NumberOrString::Number(0),
                                error: ErrorData::internal_error(
                                    format!(
                                        "the tool exists, but the server is not enabled: {}",
                                        v
                                    ),
                                    None,
                                ),
                            }));

                            return Ok(());
                        }
                    };

                    match res.take_body() {
                        ResBody::Once(v) => {
                            let result = serde_json::from_slice::<APIResponse<Value>>(&v)?;

                            if result.code == 200 {
                                res.render(Json(JsonRpcResponse {
                                    jsonrpc: JsonRpcVersion2_0,
                                    id: NumberOrString::Number(0),
                                    result: CallToolResult::success(vec![Content::text(
                                        result.data.unwrap().to_string(),
                                    )]),
                                }));
                            } else {
                                res.render(Json(JsonRpcResponse {
                                    jsonrpc: JsonRpcVersion2_0,
                                    id: NumberOrString::Number(0),
                                    result: CallToolResult::error(vec![Content::text(result.msg)]),
                                }));
                            }
                        }
                        ResBody::Error(v) => {
                            res.render(Json(JsonRpcResponse {
                                jsonrpc: JsonRpcVersion2_0,
                                id: NumberOrString::Number(0),
                                result: CallToolResult::error(vec![Content::text(v.to_string())]),
                            }));
                        }
                        _ => anyhow::bail!("take_body"),
                    }
                }
                v => {
                    res.render(Json(JsonRpcError {
                        jsonrpc: JsonRpcVersion2_0,
                        id: NumberOrString::Number(0),
                        error: ErrorData::invalid_params(
                            format!("the tool does not exist: {}", v),
                            None,
                        ),
                    }));
                }
            };
        }
        v => res.render(Json(JsonRpcError {
            jsonrpc: JsonRpcVersion2_0,
            id: NumberOrString::Number(0),
            error: ErrorData::new(ErrorCode::METHOD_NOT_FOUND, v.method().to_string(), None),
        })),
    }

    Ok(())
}

#[handler]
fn default_error_handler(res: &mut Response) {
    if let ResBody::Error(v) = &res.body {
        res.render(Json(APIResponse {
            code: v.code.as_u16(),
            msg: v.to_string(),
            data: <Option<String>>::None,
        }));
    } else {
        res.render(Text::Plain(""));
    }
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    tracing_subscriber::fmt()
        .with_ansi(enable_ansi_support().is_ok())
        .init();

    if !(args.ocr || args.old) && !args.det && !args.slide {
        warn!("no enabled features, default enabled all features");

        args.ocr = true;
        args.old = true;
        args.det = true;
        args.slide = true;
    }

    ARGS.set(args.clone()).unwrap();

    init_ocr(&args);

    if args.slide {
        info!("slide enabled successfully");
    }

    if args.mcp {
        info!("mcp enabled successfully");
    }

    if args.only_mcp {
        info!("only mcp enabled successfully");
    }

    let mut router = Router::new();

    router = router
        .hoop(salvo::prelude::Logger::new())
        .push(Router::with_path("/status").get(route_status));

    if (args.ocr || args.old) && !args.only_mcp {
        router = router.push(Router::with_path("/ocr").post(route_ocr));
    }

    if args.det && !args.only_mcp {
        router = router.push(Router::with_path("/det").post(route_det));
    }

    if args.slide && !args.only_mcp {
        router = router
            .push(Router::with_path("/slide-match").post(route_slide_match))
            .push(Router::with_path("/slide-comparison").post(route_slide_comparison));
    }

    if args.mcp || args.only_mcp {
        router = router.push(Router::with_path("/mcp").post(route_mcp))
    }

    let docs = OpenApi::new("ddddocr api", "0.0.1").merge_router(&router);

    let router = router
        .unshift(docs.into_router("/docs.json"))
        .unshift(SwaggerUi::new("/docs.json").into_router("/docs"));

    let service = Service::new(router).catcher(Catcher::default().hoop(default_error_handler));

    request::set_global_secure_max_size(50 * 1024 * 1024);

    let acceptor = TcpListener::new(args.address);

    if let Some(v) = &args.acme {
        Server::new(acceptor.acme().add_domain(v).bind().await)
            .serve(service)
            .await;
    } else {
        Server::new(acceptor.bind().await).serve(service).await;
    }
}

fn ocr_charset_range(args: &Args) -> Option<CharsetRange> {
    args.ocr_charset_range.as_ref().map(|v| match v.as_str() {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" => {
            CharsetRange::from(v.parse::<i32>().unwrap())
        }
        v => CharsetRange::from(v),
    })
}

#[cfg(feature = "inline-model")]
fn init_ocr(args: &Args) {
    let ocr_charset_range = ocr_charset_range(args);

    if args.ocr {
        let mut ddddocr = ddddocr_classification().unwrap();

        if let Some(v) = ocr_charset_range {
            ddddocr.set_ranges(v)
        };

        OCR.set(ddddocr).unwrap();

        info!("ocr enabled successfully");
    } else if args.old {
        let mut ddddocr = ddddocr_classification_old().unwrap();

        if let Some(v) = ocr_charset_range {
            ddddocr.set_ranges(v)
        };

        OCR.set(ddddocr).unwrap();

        info!("old enabled successfully");
    }

    if args.det {
        DET.set(ddddocr_detection().unwrap()).unwrap();

        info!("det enabled successfully");
    }
}

#[cfg(not(feature = "inline-model"))]
fn init_ocr(args: &Args) {
    use std::fs::{read, read_to_string};
    use std::path::PathBuf;
    use std::str::FromStr;

    let ocr_charset_range = ocr_charset_range(args);

    if args.ocr || args.old {
        let mut path = PathBuf::from(args.ocr_path.clone());

        let model = read(&path).expect("failed to open the ocr model file");

        path.set_extension("json");

        let charset = read_to_string(path).expect("failed to open the ocr charset file");

        let mut ddddocr = Ddddocr::new(
            &model,
            Charset::from_str(&charset).expect("failed to parse charset"),
        )
        .unwrap();

        if let Some(v) = ocr_charset_range {
            ddddocr.set_ranges(v)
        };

        OCR.set(ddddocr).unwrap();

        if args.ocr {
            info!("ocr enabled successfully");
        } else if args.old {
            info!("old enabled successfully");
        }
    }

    if args.det {
        let model = read(args.det_path.clone()).expect("failed to open the det model file");

        DET.set(Ddddocr::new_model(&model).unwrap()).unwrap();

        info!("det enabled successfully");
    }
}
