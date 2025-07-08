/// 初始化内容识别。
pub fn ddddocr_classification() -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new(
        include_bytes!("../model/common.onnx"),
        serde_json::from_str(include_str!("../model/common.json")).unwrap(),
    )
}

/// 初始化内容识别。
#[cfg(feature = "cuda")]
pub fn ddddocr_classification_cuda(device_id: i32) -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new_cuda(
        include_bytes!("../model/common.onnx"),
        serde_json::from_str(include_str!("../model/common.json")).unwrap(),
        device_id,
    )
}

/// 使用旧模型初始化内容识别。
pub fn ddddocr_classification_old() -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new(
        include_bytes!("../model/common_old.onnx"),
        serde_json::from_str(include_str!("../model/common_old.json")).unwrap(),
    )
}

/// 使用旧模型初始化内容识别。
#[cfg(feature = "cuda")]
pub fn ddddocr_classification_old_cuda(device_id: i32) -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new_cuda(
        include_bytes!("../model/common_old.onnx"),
        serde_json::from_str(include_str!("../model/common_old.json")).unwrap(),
        device_id,
    )
}

/// 初始化目标检测。
pub fn ddddocr_detection() -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new_model(include_bytes!("../model/common_det.onnx"))
}

/// 初始化目标检测。
#[cfg(feature = "cuda")]
pub fn ddddocr_detection_cuda(device_id: i32) -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new_model_cuda(include_bytes!("../model/common_det.onnx"), device_id)
}

/// 滑块匹配。
pub fn slide_match<I1, I2>(target_image: I1, background_image: I2) -> anyhow::Result<SlideBBox>
where
    I1: AsRef<[u8]>,
    I2: AsRef<[u8]>,
{
    let target_image = image::load_from_memory(target_image.as_ref())?;
    let background_image = image::load_from_memory(background_image.as_ref())?;

    anyhow::ensure!(
        background_image.width() >= target_image.width(),
        "背景图片的宽度必须大于等于目标图片的宽度"
    );

    anyhow::ensure!(
        background_image.height() >= target_image.height(),
        "背景图片的高度必须大于等于目标图片的高度"
    );

    let target_image = target_image.to_rgba8();

    // 裁剪图片，只保留不透明部分
    let width = target_image.width();
    let height = target_image.height();
    let mut start_x = width;
    let mut start_y = height;
    let mut end_x = 0;
    let mut end_y = 0;

    for x in 0..width {
        for y in 0..height {
            let p = target_image[(x, y)];

            if p[3] != 0 {
                if x < start_x {
                    start_x = x;
                }

                if y < start_y {
                    start_y = y;
                }

                if x > end_x {
                    end_x = x;
                }

                if y > end_y {
                    end_y = y;
                }
            }
        }
    }

    let cropped_image = if start_x > end_x || start_y > end_y {
        // 没有任何不透明的像素
        target_image
    } else {
        image::imageops::crop_imm(
            &target_image,
            start_x,
            start_y,
            end_x - start_x + 1,
            end_y - start_y + 1,
        )
        .to_image()
    };

    // 图片转换到灰度图
    let target_image = image::imageops::grayscale(&cropped_image);

    // 使用 canny 进行边缘检测。然后对背景图片进行同样的处理
    // 接着，使用 match_template 函数进行模板匹配，得到匹配结果矩阵
    // 然后使用 find_extremes 函数找到结果矩阵中的最大值和最小值
    // 并得到最大值所在的位置 loc，根据目标图片的大小和 loc 计算出目标物体的位置信息
    let target_image = imageproc::edges::canny(&target_image, 100.0, 200.0);
    let background_image = imageproc::edges::canny(&background_image.to_luma8(), 100.0, 200.0);
    let result =
        imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
            &background_image,
            &target_image,
            imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
        ));

    Ok(SlideBBox {
        target_x: start_x,
        target_y: start_y,
        x1: result.max_value_location.0,
        y1: result.max_value_location.1,
        x2: result.max_value_location.0 + target_image.width(),
        y2: result.max_value_location.1 + target_image.height(),
    })
}

/// 滑块匹配。
pub fn slide_match_with_path<P1, P2>(
    target_image: P1,
    background_image: P2,
) -> anyhow::Result<SlideBBox>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    slide_match(
        std::fs::read(target_image)?,
        std::fs::read(background_image)?,
    )
}

/// 如果小图无过多背景部分，可以使用简单滑块匹配。
pub fn simple_slide_match<I1, I2>(
    target_image: I1,
    background_image: I2,
) -> anyhow::Result<SlideBBox>
where
    I1: AsRef<[u8]>,
    I2: AsRef<[u8]>,
{
    let target_image = image::load_from_memory(target_image.as_ref())?;
    let background_image = image::load_from_memory(background_image.as_ref())?;

    anyhow::ensure!(
        background_image.width() >= target_image.width(),
        "背景图片的宽度必须大于等于目标图标的宽度"
    );

    anyhow::ensure!(
        background_image.height() >= target_image.height(),
        "背景图片的高度必须大于等于目标图标的高度"
    );

    // 使用 canny 进行边缘检测。然后对背景图片进行同样的处理
    // 接着，使用 match_template 函数进行模板匹配，得到匹配结果矩阵
    // 然后使用 find_extremes 函数找到结果矩阵中的最大值和最小值
    // 并得到最大值所在的位置 loc，根据目标图片的大小和 loc 计算出目标物体的位置信息
    let target_image = imageproc::edges::canny(&target_image.to_luma8(), 100.0, 200.0);
    let background_image = imageproc::edges::canny(&background_image.to_luma8(), 100.0, 200.0);
    let result =
        imageproc::template_matching::find_extremes(&imageproc::template_matching::match_template(
            &background_image,
            &target_image,
            imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
        ));

    Ok(SlideBBox {
        target_x: 0,
        target_y: 0,
        x1: result.max_value_location.0,
        y1: result.max_value_location.1,
        x2: result.max_value_location.0 + target_image.width(),
        y2: result.max_value_location.1 + target_image.height(),
    })
}

/// 如果小图无过多背景部分，可以使用简单滑块匹配。
pub fn simple_slide_match_with_path<P1, P2>(
    target_image: P1,
    background_image: P2,
) -> anyhow::Result<SlideBBox>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    simple_slide_match(
        std::fs::read(target_image)?,
        std::fs::read(background_image)?,
    )
}

/// 坑位匹配。
pub fn slide_comparison<I1, I2>(
    target_image: I1,
    background_image: I2,
) -> anyhow::Result<(u32, u32)>
where
    I1: AsRef<[u8]>,
    I2: AsRef<[u8]>,
{
    let target_image = image::load_from_memory(target_image.as_ref())?;
    let background_image = image::load_from_memory(background_image.as_ref())?;

    anyhow::ensure!(
        target_image.width() == background_image.width()
            && target_image.height() == background_image.height(),
        "图片尺寸不相等"
    );

    let image = image::RgbImage::from_vec(
        target_image.width(),
        target_image.height(),
        target_image
            .as_bytes()
            .iter()
            .zip(background_image.as_bytes().iter())
            .map(|(a, b)| if a.abs_diff(*b) > 80 { 255 } else { 0 })
            .collect(),
    )
    .unwrap();

    let mut start_x = 0;
    let mut start_y = 0;

    for i in 0..image.width() {
        let mut count = 0;

        for j in 0..image.height() {
            let pixel = image[(i, j)];

            if pixel != image::Rgb([0, 0, 0]) {
                count += 1;
            }

            if count >= 5 && start_y == 0 {
                start_y = j - 5;
            }
        }

        if count >= 5 {
            start_x = i + 2;
            break;
        }
    }

    Ok((start_x, start_y))
}

/// 坑位匹配。
pub fn slide_comparison_with_path<P1, P2>(
    target_image: P1,
    background_image: P2,
) -> anyhow::Result<(u32, u32)>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    slide_comparison(
        std::fs::read(target_image)?,
        std::fs::read(background_image)?,
    )
}

/// 判断是否为自定义模型。
pub fn is_diy<MODEL>(model: MODEL) -> bool
where
    MODEL: AsRef<[u8]>,
{
    // 比较 common.onnx 和 common_old.onnx 的 sha256
    let sha256 = sha256::digest(model.as_ref());

    sha256 != "33b5cd351ee94e73a6bf8fa18c415ed8b819b3ffd342e267c30d8ad8334e34e8"
        && sha256 != "b8f2ad9cbc1f2e3922a6cb9459e30824e7e2467f3fb4fd61420640e34ea0bf68"
}

/// 将图片的透明部分用白色填充。
fn png_rgba_black_preprocess(image: &image::DynamicImage) -> image::DynamicImage {
    let (width, height) = image::GenericImageView::dimensions(image);
    let mut new_image = image::DynamicImage::new_rgb8(width, height);

    for y in 0..height {
        for x in 0..width {
            let rgba_pixel = image::GenericImageView::get_pixel(image, x, y);

            let rgb_pixel = if rgba_pixel[3] == 0 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([rgba_pixel[0], rgba_pixel[1], rgba_pixel[2], rgba_pixel[3]])
            };

            image::GenericImage::put_pixel(&mut new_image, x, y, rgb_pixel);
        }
    }

    new_image
}

/// 内容识别需要用到的配置。
///
/// `../model/common_charset.json`
/// `../model/common_old_charset.json`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Charset {
    /// 是否为 cnn 模型。
    pub word: bool,

    /// 宽度，高度，如果宽度为 -1，则自动调整，高度必须为 16 的倍数。
    pub image: [i64; 2],

    /// 通道数量。
    pub channel: i64,

    /// 字符集。
    pub charset: Vec<String>,
}

impl std::str::FromStr for Charset {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

/// 文字坐标。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BBox {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

/// 滑块坐标。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SlideBBox {
    /// 透明部分的 x 偏移。
    pub target_x: u32,

    /// 透明部分的 y 偏移。
    pub target_y: u32,

    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

/// 字符集和概率。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterProbability {
    pub text: Option<String>,
    pub charset: Vec<String>,
    pub probability: Vec<Vec<f32>>,
    pub confidence: Option<f64>,
}

/// 有时候只想获取 probability，而不获取 text 和 confidence。
impl CharacterProbability {
    pub fn get_text(&mut self) -> &str {
        self.text.get_or_insert_with(|| {
            let mut s = String::new();

            for i in &self.probability {
                let (n, _) = i
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.total_cmp(b))
                    .unwrap();

                s += &self.charset[n];
            }

            return s;
        })
    }

    pub fn get_confidence(&mut self) -> f64 {
        *self.confidence.get_or_insert_with(|| {
            let mut max_sum = 0.0;
            let mut count = 0usize;

            for i in &self.probability {
                if let Some(v) = i.iter().fold(None, |acc: Option<f64>, &i| {
                    acc.map_or(Some(i as f64), |max| Some(max.max(i as f64)))
                }) {
                    max_sum += v as f64;
                    count += 1;
                }
            }

            if count == 0 {
                0.0
            } else {
                max_sum / count as f64
            }
        })
    }
}

pub trait MapJson {
    fn json(&self) -> String;
}

impl MapJson for BBox {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MapJson for Vec<BBox> {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MapJson for SlideBBox {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MapJson for (u32, u32) {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MapJson for CharacterProbability {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MapJson for Vec<(BBox, String)> {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub trait MapBBox {
    fn to_tuple(&self) -> Vec<(u32, u32, u32, u32)>;

    fn to_vec(&self) -> Vec<Vec<u32>>;
}

impl MapBBox for Vec<BBox> {
    fn to_tuple(&self) -> Vec<(u32, u32, u32, u32)> {
        self.iter()
            .map(|v| (v.x1, v.y1, v.x2, v.y2))
            .collect::<Vec<_>>()
    }

    fn to_vec(&self) -> Vec<Vec<u32>> {
        self.iter()
            .map(|v| vec![v.x1, v.y1, v.x2, v.y2])
            .collect::<Vec<_>>()
    }
}

lazy_static::lazy_static! {
    static ref _STATIC: (Vec<u32>, Vec<u32>) = {
        let mut grids = Vec::new();
        let mut expanded_strides = Vec::new();
        let hsizes = STRIDES.iter().map(|v| MODEL_HEIGHT / v).collect::<Vec<_>>();
        let wsizes = STRIDES.iter().map(|v| MODEL_WIDTH / v).collect::<Vec<_>>();

        fn meshgrid(x: u32, y: u32) -> Vec<u32> {
            let mut result = vec![0; (x * y * 2) as usize];

            for i in 0..x {
                for j in 0..y {
                    let index = ((i * x + j) * 2) as usize;
                    result[index] = j;
                    result[index + 1] = i;
                }
            }
            result
        }

        for (i, v) in STRIDES.iter().enumerate() {
            let hsize = hsizes[i];
            let wsize = wsizes[i];
            let grid = meshgrid(hsize, wsize);
            let expanded_stride = vec![*v; (hsize * wsize) as usize];

            grids.extend(grid);
            expanded_strides.extend(expanded_stride);
        }

        (grids, expanded_strides)
    };

    static ref GRIDS: Vec<u32> = unsafe { std::mem::transmute_copy(&_STATIC.0) };

    static ref EXPANDED_STRIDES: Vec<u32> =  unsafe { std::mem::transmute_copy(&_STATIC.1) };
}

const NMS_THR: f32 = 0.45;
const SCORE_THR: f32 = 0.1;
const MODEL_WIDTH: u32 = 416;
const MODEL_HEIGHT: u32 = 416;
const STRIDES: [u32; 3] = [8, 16, 32];

/// 字符集范围。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CharsetRange {
    /// 纯整数 0-9。
    Digit,

    /// 纯小写字母 a-z。
    Lowercase,

    /// 纯大写字母 a-z。
    Uppercase,

    /// 小写字母 a-z + 大写字母 A-Z。
    LowercaseUppercase,

    /// 小写字母 a-z + 整数 0-9。
    LowercaseDigit,

    /// 大写字母 A-Z + 整数 0-9。
    UppercaseDigit,

    /// 小写字母 a-z + 大写字母 A-Z + 整数 0-9。
    LowercaseUppercaseDigit,

    /// 默认字符集，删除小写字母 a-z、大写字母 A-Z、整数 0-9。
    DefaultCharsetLowercaseUppercaseDigit,

    /// 自定义字符集，例如 `0123456789+-x/=`。
    Other(String),

    /// 直接设置字符集，即 set_ranges 处理后的结果。
    Charset(Vec<String>),
}

impl From<i32> for CharsetRange {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Digit,
            1 => Self::Lowercase,
            2 => Self::Uppercase,
            3 => Self::LowercaseUppercase,
            4 => Self::LowercaseDigit,
            5 => Self::UppercaseDigit,
            6 => Self::LowercaseUppercaseDigit,
            7 => Self::DefaultCharsetLowercaseUppercaseDigit,
            _ => panic!("invalid charset range: {}", value),
        }
    }
}

impl From<&str> for CharsetRange {
    fn from(value: &str) -> Self {
        CharsetRange::Other(value.to_string())
    }
}

impl From<String> for CharsetRange {
    fn from(value: String) -> Self {
        CharsetRange::Other(value)
    }
}

impl From<&String> for CharsetRange {
    fn from(value: &String) -> Self {
        CharsetRange::Other(value.clone())
    }
}

impl From<Vec<String>> for CharsetRange {
    fn from(value: Vec<String>) -> Self {
        CharsetRange::Charset(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    /// 红色。
    Red,

    /// 蓝色。
    Blue,

    /// 绿色。
    Green,

    /// 黄色。
    Yellow,

    /// 橙色。
    Orange,

    /// 紫色。
    Purple,

    /// 青色。
    Cyan,

    /// 黑色。
    Black,

    /// 白色。
    White,

    /// 灰色。
    Gray,
}

impl<T: AsRef<str>> From<T> for Color {
    fn from(value: T) -> Self {
        match value.as_ref().to_ascii_lowercase().as_str() {
            "red" => Color::Red,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "orange" => Color::Orange,
            "purple" => Color::Purple,
            "cyan" => Color::Cyan,
            "black" => Color::Black,
            "white" => Color::White,
            "gray" => Color::Gray,
            _ => panic!("unknown color: {}", value.as_ref()),
        }
    }
}

pub trait IntoHsvRange {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))>;
}

impl IntoHsvRange for Color {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        match self {
            Color::Red => vec![
                ((0, 50, 50), (10, 255, 255)),
                ((170, 50, 50), (180, 255, 255)),
            ],
            Color::Blue => vec![((100, 50, 50), (140, 255, 255))],
            Color::Green => vec![((40, 50, 50), (80, 255, 255))],
            Color::Yellow => vec![((20, 50, 50), (40, 255, 255))],
            Color::Orange => vec![((10, 50, 50), (20, 255, 255))],
            Color::Purple => vec![((140, 50, 50), (170, 255, 255))],
            Color::Cyan => vec![((80, 50, 50), (100, 255, 255))],
            Color::Black => vec![((0, 0, 0), (180, 255, 30))],
            Color::White => vec![((0, 0, 200), (180, 30, 255))],
            Color::Gray => vec![((0, 0, 30), (180, 30, 200))],
        }
    }
}

/// 颜色过滤。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ColorFilter {
    /// HSV 范围，每个元素是一个 (min_hsv, max_hsv) 的元组。
    /// 例如: `[((0, 50, 50), (10, 255, 255))]`。
    HSVRanges(Vec<((u8, u8, u8), (u8, u8, u8))>),

    /// 颜色范围，例如: `["red", "blue"]`。
    ColorRanges(Vec<Color>),

    /// 单个颜色。
    Color(Color),
}

impl IntoHsvRange for ColorFilter {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        match self {
            ColorFilter::HSVRanges(v) => v,
            ColorFilter::ColorRanges(v) => ColorFilter::from(v).into_hsv_ranges(),
            ColorFilter::Color(v) => v.into_hsv_ranges(),
        }
    }
}

impl IntoHsvRange for &str {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        Color::from(self).into_hsv_ranges()
    }
}

impl IntoHsvRange for String {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        Color::from(self).into_hsv_ranges()
    }
}

impl IntoHsvRange for &String {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        Color::from(self).into_hsv_ranges()
    }
}

impl IntoHsvRange for ((u8, u8, u8), (u8, u8, u8)) {
    fn into_hsv_ranges(self) -> Vec<((u8, u8, u8), (u8, u8, u8))> {
        vec![self]
    }
}

impl From<&str> for ColorFilter {
    fn from(value: &str) -> Self {
        Color::from(value).into_hsv_ranges().into()
    }
}

impl From<String> for ColorFilter {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&String> for ColorFilter {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}

impl From<Color> for ColorFilter {
    fn from(value: Color) -> Self {
        value.into_hsv_ranges().into()
    }
}

impl From<&[Color]> for ColorFilter {
    fn from(value: &[Color]) -> Self {
        value.to_vec().into()
    }
}

impl From<&[&str]> for ColorFilter {
    fn from(value: &[&str]) -> Self {
        value
            .iter()
            .map(|v| Color::from(*v))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<T, const N: usize> From<[T; N]> for ColorFilter
where
    T: IntoHsvRange,
{
    fn from(value: [T; N]) -> Self {
        value
            .into_iter()
            .map(|v| v.into_hsv_ranges())
            .flatten()
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for ColorFilter
where
    T: IntoHsvRange + Clone,
{
    fn from(value: &'a [T; N]) -> Self {
        value
            .into_iter()
            .map(|v| v.clone().into_hsv_ranges())
            .flatten()
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<&[String]> for ColorFilter {
    fn from(values: &[String]) -> Self {
        values.to_vec().into()
    }
}

impl From<Vec<&str>> for ColorFilter {
    fn from(value: Vec<&str>) -> Self {
        value
            .into_iter()
            .map(|v| Color::from(v).into_hsv_ranges())
            .flatten()
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<Vec<String>> for ColorFilter {
    fn from(value: Vec<String>) -> Self {
        value
            .iter()
            .map(|v| Color::from(v.as_str()).into_hsv_ranges())
            .flatten()
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<Vec<Color>> for ColorFilter {
    fn from(value: Vec<Color>) -> Self {
        value
            .into_iter()
            .map(|v| v.into_hsv_ranges())
            .flatten()
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<Vec<((u8, u8, u8), (u8, u8, u8))>> for ColorFilter {
    fn from(value: Vec<((u8, u8, u8), (u8, u8, u8))>) -> Self {
        ColorFilter::HSVRanges(value)
    }
}

impl ColorFilter {
    /// 过滤颜色，例如 ColorFilter::from("green").filter(image) 表示只保留绿色。
    pub fn filter<I>(&self, image: I) -> anyhow::Result<image::DynamicImage>
    where
        I: AsRef<[u8]>,
    {
        let image = image::load_from_memory(image.as_ref())?.to_rgb8();
        let (width, height) = image.dimensions();
        let mut array = ndarray::Array3::<u8>::zeros((height as usize, width as usize, 3));

        for (x, y, pixel) in image.enumerate_pixels() {
            let y = y as usize;
            let x = x as usize;
            array[[y, x, 0]] = pixel[0];
            array[[y, x, 1]] = pixel[1];
            array[[y, x, 2]] = pixel[2];
        }

        let mut hsv = ndarray::Array3::<u8>::zeros((height as usize, width as usize, 3));

        for y in 0..height as usize {
            for x in 0..width as usize {
                let r = array[[y, x, 0]] as f32 / 255.0;
                let g = array[[y, x, 1]] as f32 / 255.0;
                let b = array[[y, x, 2]] as f32 / 255.0;
                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                let v = max;
                let delta = max - min;
                let s = if max == 0.0 { 0.0 } else { delta / max };
                let h_deg = if delta == 0.0 {
                    0.0
                } else if max == r {
                    60.0 * (((g - b) / delta) % 6.0)
                } else if max == g {
                    60.0 * (((b - r) / delta) + 2.0)
                } else {
                    60.0 * (((r - g) / delta) + 4.0)
                };

                let h_deg = if h_deg < 0.0 { h_deg + 360.0 } else { h_deg };

                hsv[[y, x, 0]] = (h_deg / 2.0).round().min(180.0) as u8;
                hsv[[y, x, 1]] = (s * 255.0).round().min(255.0) as u8;
                hsv[[y, x, 2]] = (v * 255.0).round().min(255.0) as u8;
            }
        }

        let mut mask = ndarray::Array2::<bool>::from_elem((height as usize, width as usize), false);

        let ranges = match self {
            ColorFilter::HSVRanges(v) => v,
            ColorFilter::ColorRanges(v) => &v
                .iter()
                .flat_map(|v| v.into_hsv_ranges())
                .collect::<Vec<_>>(),
            ColorFilter::Color(v) => &v.into_hsv_ranges(),
        };

        for (lower, upper) in ranges {
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let h = hsv[[y, x, 0]];
                    let s = hsv[[y, x, 1]];
                    let v = hsv[[y, x, 2]];

                    if h >= lower.0
                        && h <= upper.0
                        && s >= lower.1
                        && s <= upper.1
                        && v >= lower.2
                        && v <= upper.2
                    {
                        mask[[y, x]] = true;
                    }
                }
            }
        }

        let mut result: image::RgbImage = image::ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let yi = y as usize;
                let xi = x as usize;
                let pixel = if mask[[yi, xi]] {
                    image::Rgb([array[[yi, xi, 0]], array[[yi, xi, 1]], array[[yi, xi, 2]]])
                } else {
                    image::Rgb([255, 255, 255])
                };

                result.put_pixel(x, y, pixel);
            }
        }

        Ok(image::DynamicImage::ImageRgb8(result))
    }
}

#[derive(Debug)]
pub struct Ddddocr<'a> {
    diy: bool,
    session: ort::Session,
    charset: Option<std::borrow::Cow<'a, Charset>>,
    charset_range: Vec<String>,
}

unsafe impl<'a> Send for Ddddocr<'a> {}
unsafe impl<'a> Sync for Ddddocr<'a> {}

/// 因为自带模型和自定义模型的参数不同，所以在创建模型的时候会自动判断是否为自定义模型。
impl<'a> Ddddocr<'a> {
    /// 设置运行库的路径。
    ///
    /// - Unix: `/etc/.../libonnxruntime.so`
    /// - Windows: `C:\\Program Files\\...\\onnxruntime.dll`
    ///
    /// 不需要手动调用此函数，因为程序会自动寻找运行库路径。
    #[cfg(feature = "load-dynamic")]
    pub fn set_onnxruntime_path<P>(path: P) -> anyhow::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();

        let save = std::panic::take_hook();

        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(|| {
            ort::init_from(path.to_string_lossy().to_string())
                .commit()
                .unwrap()
        });

        std::panic::set_hook(save);

        result.map_err(|v| {
            anyhow::anyhow!(
                "{}",
                v.downcast::<String>().unwrap_or(Box::new(format!(
                    "failed to load the runtime library: {}",
                    path.display()
                )))
            )
        })
    }

    /// 从内存加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    pub fn new<MODEL>(model: MODEL, charset: Charset) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ort::Session::builder()?.commit_from_memory(model.as_ref())?,
            charset: Some(std::borrow::Cow::Owned(charset)),
            charset_range: Vec::new(),
        })
    }

    /// 从内存加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    pub fn new_ref<MODEL>(model: MODEL, charset: &'a Charset) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ort::Session::builder()?.commit_from_memory(model.as_ref())?,
            charset: Some(std::borrow::Cow::Borrowed(charset)),
            charset_range: Vec::new(),
        })
    }

    /// 从内存加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_cuda<MODEL>(model: MODEL, charset: Charset, device_id: i32) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        let builder = ort::Session::builder()?;

        let cuda = ort::CUDAExecutionProvider::default()
            .with_device_id(device_id)
            .with_arena_extend_strategy(ort::ArenaExtendStrategy::NextPowerOfTwo)
            .with_memory_limit(2 * 1024 * 1024 * 1024)
            .with_conv_algorithm_search(ort::CUDAExecutionProviderCuDNNConvAlgoSearch::Exhaustive)
            .with_copy_in_default_stream(true);

        if !ort::ExecutionProvider::is_available(&cuda)? {
            anyhow::bail!("Please compile ONNX Runtime with CUDA!")
        }

        ort::ExecutionProvider::register(&cuda, &builder)?;

        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: builder.commit_from_memory(model.as_ref())?,
            charset: Some(std::borrow::Cow::Owned(charset)),
            charset_range: Vec::new(),
        })
    }

    /// 从内存加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_cuda_ref<MODEL>(
        model: MODEL,
        charset: &'a Charset,
        device_id: i32,
    ) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        let builder = ort::Session::builder()?;

        let cuda = ort::CUDAExecutionProvider::default()
            .with_device_id(device_id)
            .with_arena_extend_strategy(ort::ArenaExtendStrategy::NextPowerOfTwo)
            .with_memory_limit(2 * 1024 * 1024 * 1024)
            .with_conv_algorithm_search(ort::CUDAExecutionProviderCuDNNConvAlgoSearch::Exhaustive)
            .with_copy_in_default_stream(true);

        if !ort::ExecutionProvider::is_available(&cuda)? {
            anyhow::bail!("Please compile ONNX Runtime with CUDA!")
        }

        ort::ExecutionProvider::register(&cuda, &builder)?;

        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: builder.commit_from_memory(model.as_ref())?,
            charset: Some(std::borrow::Cow::Borrowed(charset)),
            charset_range: Vec::new(),
        })
    }

    /// 从内存加载模型，只能使用目标检测，使用内容识别会恐慌。
    pub fn new_model<MODEL>(model: MODEL) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ort::Session::builder()?.commit_from_memory(model.as_ref())?,
            charset: None,
            charset_range: Vec::new(),
        })
    }

    /// 从内存加载模型，只能使用目标检测，使用内容识别会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_model_cuda<MODEL>(model: MODEL, device_id: i32) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        let builder = ort::Session::builder()?;

        let cuda = ort::CUDAExecutionProvider::default()
            .with_device_id(device_id)
            .with_arena_extend_strategy(ort::ArenaExtendStrategy::NextPowerOfTwo)
            .with_memory_limit(2 * 1024 * 1024 * 1024)
            .with_conv_algorithm_search(ort::CUDAExecutionProviderCuDNNConvAlgoSearch::Exhaustive)
            .with_copy_in_default_stream(true);

        if !ort::ExecutionProvider::is_available(&cuda)? {
            anyhow::bail!("Please compile ONNX Runtime with CUDA!")
        }

        ort::ExecutionProvider::register(&cuda, &builder)?;

        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: builder.commit_from_memory(model.as_ref())?,
            charset: None,
            charset_range: Vec::new(),
        })
    }

    /// 从文件加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    pub fn with_model_charset<PATH1, PATH2>(model: PATH1, charset: PATH2) -> anyhow::Result<Self>
    where
        PATH1: AsRef<std::path::Path>,
        PATH2: AsRef<std::path::Path>,
    {
        Self::new(
            std::fs::read(model)?,
            serde_json::from_str(&std::fs::read_to_string(charset)?)?,
        )
    }

    /// 从文件加载模型和字符集，只能使用内容识别，使用目标检测会恐慌。
    #[cfg(feature = "cuda")]
    pub fn with_model_charset_cuda<PATH1, PATH2>(
        model: PATH1,
        charset: PATH2,
        device_id: i32,
    ) -> anyhow::Result<Self>
    where
        PATH1: AsRef<std::path::Path>,
        PATH2: AsRef<std::path::Path>,
    {
        Self::new_cuda(
            std::fs::read(model)?,
            serde_json::from_str(&std::fs::read_to_string(charset)?)?,
            device_id,
        )
    }

    /// 从文件加载模型，只能使用目标检测，使用内容识别会恐慌。
    pub fn with_model<P>(model: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Self::new_model(std::fs::read(model)?)
    }

    /// 从文件加载模型，只能使用目标检测，使用内容识别会恐慌。
    #[cfg(feature = "cuda")]
    pub fn with_model_cuda<P>(model: P, device_id: i32) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Self::new_model_cuda(std::fs::read(model)?, device_id)
    }

    /// 根据给定 ranges 计算字符集范围。
    pub fn calc_ranges<R>(&self, ranges: R) -> Vec<String>
    where
        R: Into<CharsetRange>,
    {
        let mut new_charset = match ranges.into() {
            CharsetRange::Digit => "0123456789"
                .chars()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            CharsetRange::Lowercase => "abcdefghijklmnopqrstuvwxyz"
                .chars()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            CharsetRange::Uppercase => "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            CharsetRange::LowercaseUppercase => {
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .chars()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
            }
            CharsetRange::LowercaseDigit => "abcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            CharsetRange::UppercaseDigit => "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                .chars()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            CharsetRange::LowercaseUppercaseDigit => {
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    .chars()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
            }
            CharsetRange::DefaultCharsetLowercaseUppercaseDigit => {
                let delete_range = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    .chars()
                    .collect::<Vec<char>>();

                (&**self
                    .charset
                    .as_ref()
                    .expect("only the ocr model can be used"))
                    .charset
                    .clone()
                    .into_iter()
                    .filter(|v| v.chars().all(|c| !delete_range.contains(&c)))
                    .collect::<Vec<String>>()
            }
            CharsetRange::Other(v) => v.chars().map(|v| v.to_string()).collect::<Vec<String>>(),
            CharsetRange::Charset(v) => return v,
        };

        // 去重 + 补空字符串
        new_charset = new_charset
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<String>>();

        new_charset.push("".to_string());

        new_charset
    }

    /// 限定 classification_probability 的字符范围，只能使用内容识别，使用目标检测会恐慌。
    pub fn set_ranges<R>(&mut self, ranges: R)
    where
        R: Into<CharsetRange>,
    {
        self.charset_range = self.calc_ranges(ranges)
    }

    /// 内容识别，返回全字符表的概率，可以通过 `set_ranges` 限定字符范围，仅限于使用官方模型。
    pub fn classification_probability<I>(&self, image: I) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
    {
        self.classification_probability_with_options(image, false, None, None)
    }

    /// 内容识别，返回全字符表的概率，可以通过 `set_ranges` 限定字符范围，仅限于使用官方模型。
    /// 如果 `png_fix` 为 true，则支持透明黑色背景的 png 图片。
    pub fn classification_probability_with_png_fix<I>(
        &self,
        image: I,
        png_fix: bool,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
    {
        self.classification_probability_with_options(image, png_fix, None, None)
    }

    /// 内容识别，返回全字符表的概率，可以通过 `set_ranges` 限定字符范围，仅限于使用官方模型。
    /// 如果 `filter` 为 red，则表示只识别红色。
    pub fn classification_probability_with_filter<I, F>(
        &self,
        image: I,
        filter: F,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
        F: Into<ColorFilter>,
    {
        self.classification_probability_with_options(image, false, Some(filter.into()), None)
    }

    /// 内容识别，返回指定字符范围的概率，可通过 `ranges` 限定字符范围。
    pub fn classification_probability_with_ranges<I, R>(
        &self,
        image: I,
        ranges: R,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
        R: Into<CharsetRange>,
    {
        self.classification_probability_with_options(image, false, None, Some(ranges.into()))
    }

    /// 内容识别，返回指定字符范围的概率，可通过 `png_fix` 支持透明黑色背景的 png 图片，并通过 `ranges` 限定字符范围。
    pub fn classification_probability_with_png_fix_and_ranges<I, R>(
        &self,
        image: I,
        png_fix: bool,
        ranges: R,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
        R: Into<CharsetRange>,
    {
        self.classification_probability_with_options(image, png_fix, None, Some(ranges.into()))
    }

    /// 内容识别，返回指定字符范围的概率，可通过 `filter` 指定颜色过滤，并通过 `ranges` 限定字符范围。
    pub fn classification_probability_with_filter_and_ranges<I, F, R>(
        &self,
        image: I,
        filter: F,
        ranges: R,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
        F: Into<ColorFilter>,
        R: Into<CharsetRange>,
    {
        self.classification_probability_with_options(
            image,
            false,
            Some(filter.into()),
            Some(ranges.into()),
        )
    }

    /// 内容识别，返回全字符表的概率，可以通过 set_ranges 限定字符范围，仅限于使用官方模型。
    /// 如果 png_fix 为 true，则支持透明黑色背景的 png 图片。
    /// 如果 filter 为 red，则表示只识别红色。
    /// 如果 ranges 为 None，则使用 set_ranges 的字符范围。
    pub fn classification_probability_with_options<I>(
        &self,
        image: I,
        png_fix: bool,
        filter: Option<ColorFilter>,
        ranges: Option<CharsetRange>,
    ) -> anyhow::Result<CharacterProbability>
    where
        I: AsRef<[u8]>,
    {
        if self.diy {
            // 嘿，傻瓜，这里明明写了只能用官方模型，你是故意不看吗？发生 panic 的话自己负责哦！
            panic!("can only use the official model");
        }

        let mut _temp = Vec::new();
        let charset_ranges = match &ranges {
            Some(v) => 'a: {
                let new_charset = match v {
                    CharsetRange::Digit => "0123456789"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                    CharsetRange::Lowercase => "abcdefghijklmnopqrstuvwxyz"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                    CharsetRange::Uppercase => "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                    CharsetRange::LowercaseUppercase => {
                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                            .chars()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                    }
                    CharsetRange::LowercaseDigit => "abcdefghijklmnopqrstuvwxyz0123456789"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                    CharsetRange::UppercaseDigit => "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                        .chars()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                    CharsetRange::LowercaseUppercaseDigit => {
                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                            .chars()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                    }
                    CharsetRange::DefaultCharsetLowercaseUppercaseDigit => {
                        // 删除小写字母 a-z、大写字母 A-Z、整数 0-9
                        let delete_range =
                            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                                .chars()
                                .collect::<Vec<char>>();

                        (&**self
                            .charset
                            .as_ref()
                            .ok_or(anyhow::anyhow!("only the ocr model can be used"))?)
                            .charset
                            .clone()
                            .into_iter()
                            .filter(|v| v.chars().all(|c| !delete_range.contains(&c)))
                            .collect::<Vec<String>>()
                    }
                    CharsetRange::Other(v) => {
                        v.chars().map(|v| v.to_string()).collect::<Vec<String>>()
                    }
                    CharsetRange::Charset(v) => break 'a v,
                };

                // 去重
                _temp = new_charset
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<String>>();

                _temp.push("".to_string());

                &_temp
            }
            None => &self.charset_range,
        };

        let image = match filter {
            Some(v) => v.filter(image.as_ref())?,
            None => image::load_from_memory(image.as_ref())?,
        };

        let charset = self.charset.as_ref().unwrap();
        let word = charset.word;
        let resize = charset.image;
        let channel = charset.channel;
        let charset = &charset.charset;

        // 使用 ANTIALIAS (Lanczos3) 缩放图片
        let image = if resize[0] == -1 {
            if word {
                image.resize_exact(
                    resize[1] as u32,
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                image.resize_exact(
                    image.width() * resize[1] as u32 / image.height(),
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            }
        } else {
            image.resize_exact(
                resize[0] as u32,
                resize[1] as u32,
                image::imageops::FilterType::Lanczos3,
            )
        };

        // 设置图片的通道数为模型所需的通道数
        let image_bytes = if channel == 1 {
            image::EncodableLayout::as_bytes(image.to_luma8().as_ref()).to_vec()
        } else if png_fix {
            png_rgba_black_preprocess(&image).to_rgb8().to_vec()
        } else {
            image.to_rgb8().to_vec()
        };

        // 图片转换到张量
        let channel = channel as usize;
        let width = image.width() as usize;
        let height = image.height() as usize;
        let image = ndarray::Array::from_shape_vec((channel, height, width), image_bytes)?;
        let mut tensor = ndarray::Array::from_shape_vec(
            (1, channel, height, width),
            vec![0f32; height * width],
        )?;

        // 根据配置标准化图像张量
        for i in 0..height {
            for j in 0..width {
                let now = image[[0, i, j]] as f32;
                if self.diy {
                    // 自定义模型
                    if channel == 1 {
                        tensor[[0, 0, i, j]] = ((now / 255f32) - 0.456f32) / 0.224f32;
                    } else {
                        let r = image[[0, i, j]] as f32;
                        let g = image[[1, i, j]] as f32;
                        let b = image[[2, i, j]] as f32;
                        tensor[[0, 0, i, j]] = ((r / 255f32) - 0.485f32) / 0.229f32;
                        tensor[[0, 1, i, j]] = ((g / 255f32) - 0.456f32) / 0.224f32;
                        tensor[[0, 2, i, j]] = ((b / 255f32) - 0.406f32) / 0.225f32;
                    }
                } else {
                    tensor[[0, 0, i, j]] = ((now / 255f32) - 0.5f32) / 0.5f32;
                }
            }
        }

        let ort_outs = &self.session.run(ort::inputs![tensor]?)?;

        let ort_outs = &ort_outs[0].try_extract_tensor()?;

        // 长这样 [[[1,2,3,4]], [[1,2,3,4]], [[1,2,3,4]]]
        let ort_outs = ort_outs.mapv(|v| f32::exp(v)) / ort_outs.mapv(|v| f32::exp(v)).sum();

        // 长这样 [[1,2,3,4], [1,2,3,4], [1,2,3,4]]
        let ort_outs_sum = ort_outs.sum_axis(ndarray::Axis(2));

        // 根据形状创建一个零数组
        let mut ort_outs_probability = ndarray::Array::<f32, _>::zeros(ort_outs.raw_dim());

        for i in 0..ort_outs.shape()[0] {
            let mut a = ort_outs_probability.slice_mut(ndarray::s![i, .., ..]);
            let b = ort_outs.slice(ndarray::s![i, .., ..]);
            let c = ort_outs_sum.slice(ndarray::s![i, ..]);
            let d = &b / &c;

            a.assign(&d);
        }

        // 调用 next 后，长这样 [[1,2,3,4], [1,2,3,4], [1,2,3,4]]
        let ort_outs_probability = ort_outs_probability
            .axis_iter(ndarray::Axis(1))
            .next()
            .ok_or(anyhow::anyhow!("expect axis 1"))?;

        let mut result = Vec::new();

        // 扁平化
        for i in ort_outs_probability.axis_iter(ndarray::Axis(0)) {
            result.push(i.into_diag().to_vec());
        }

        if charset_ranges.is_empty() {
            // 返回全部字符的概率
            return Ok(CharacterProbability {
                text: None,
                charset: charset.clone(),
                probability: result,
                confidence: None,
            });
        } else {
            // 根据指定的字符范围，从模型输出的概率结果中提取对应字符的概率
            // 如果字符不在字符集中，则将其概率设置为 -1.0，表示未知字符
            let mut probability_result_index = Vec::new();

            for i in charset_ranges {
                if let Some(v) = charset.iter().position(|v| v == i) {
                    probability_result_index.push(v);
                } else {
                    probability_result_index.push(usize::MAX);
                }
            }

            let mut probability_result = Vec::new();

            for item in &result {
                let mut inner_vec = Vec::new();

                for &i in &probability_result_index {
                    if i != usize::MAX {
                        inner_vec.push(item[i]);
                    } else {
                        inner_vec.push(-1.0);
                    }
                }

                probability_result.push(inner_vec);
            }

            return Ok(CharacterProbability {
                text: None,
                charset: charset_ranges.clone(),
                probability: probability_result,
                confidence: None,
            });
        }
    }

    /// 内容识别。
    pub fn classification<I>(&self, image: I) -> anyhow::Result<String>
    where
        I: AsRef<[u8]>,
    {
        self.classification_with_options(image, false, None)
    }

    /// 内容识别。
    pub fn classification_with_path<P>(&self, path: P) -> anyhow::Result<String>
    where
        P: AsRef<std::path::Path>,
    {
        self.classification(std::fs::read(path)?)
    }

    /// 内容识别，如果 png_fix 为 true，则支持透明黑色背景的 png 图片。
    pub fn classification_with_png_fix<I>(&self, image: I, png_fix: bool) -> anyhow::Result<String>
    where
        I: AsRef<[u8]>,
    {
        self.classification_with_options(image, png_fix, None)
    }

    /// 内容识别，如果 png_fix 为 true，则支持透明黑色背景的 png 图片。
    pub fn classification_with_path_png_fix<P>(
        &self,
        path: P,
        png_fix: bool,
    ) -> anyhow::Result<String>
    where
        P: AsRef<std::path::Path>,
    {
        self.classification_with_png_fix(std::fs::read(path)?, png_fix)
    }

    /// 内容识别，如果 filter 为 red，则表示只识别红色。
    pub fn classification_with_filter<I, F>(&self, image: I, filter: F) -> anyhow::Result<String>
    where
        I: AsRef<[u8]>,
        F: Into<ColorFilter>,
    {
        self.classification_with_options(image, false, Some(filter.into()))
    }

    /// 内容识别，如果 filter 为 red，则表示只识别红色。
    pub fn classification_with_path_filter<P, F>(
        &self,
        path: P,
        filter: F,
    ) -> anyhow::Result<String>
    where
        P: AsRef<std::path::Path>,
        F: Into<ColorFilter>,
    {
        self.classification_with_filter(std::fs::read(path)?, filter)
    }

    /// 内容识别，如果 png_fix 为 true，则支持透明黑色背景的 png 图片，如果 filter 为 red，则表示只识别红色。
    pub fn classification_with_options<I>(
        &self,
        image: I,
        png_fix: bool,
        filter: Option<ColorFilter>,
    ) -> anyhow::Result<String>
    where
        I: AsRef<[u8]>,
    {
        let image = match filter {
            Some(v) => v.filter(image.as_ref())?,
            None => image::load_from_memory(image.as_ref())?,
        };

        let charset = self.charset.as_ref().unwrap();
        let word = charset.word;
        let resize = charset.image;
        let channel = charset.channel;
        let charset = &charset.charset;

        // 使用 ANTIALIAS (Lanczos3) 缩放图片
        let image = if resize[0] == -1 {
            if word {
                image.resize_exact(
                    resize[1] as u32,
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                image.resize_exact(
                    image.width() * resize[1] as u32 / image.height(),
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            }
        } else {
            image.resize_exact(
                resize[0] as u32,
                resize[1] as u32,
                image::imageops::FilterType::Lanczos3,
            )
        };

        // 设置图片的通道数为模型所需的通道数
        let image_bytes = if channel == 1 {
            image::EncodableLayout::as_bytes(image.to_luma8().as_ref()).to_vec()
        } else if png_fix {
            png_rgba_black_preprocess(&image).to_rgb8().to_vec()
        } else {
            image.to_rgb8().to_vec()
        };

        // 图片转换到张量
        let channel = channel as usize;

        let width = image.width() as usize;

        let height = image.height() as usize;

        let image = ndarray::Array::from_shape_vec((channel, height, width), image_bytes)?;

        let mut tensor = ndarray::Array::from_shape_vec(
            (1, channel, height, width),
            vec![0f32; channel * height * width],
        )?;

        // 根据配置标准化图像张量
        for i in 0..height {
            for j in 0..width {
                let now = image[[0, i, j]] as f32;

                if self.diy {
                    // 自定义模型
                    if channel == 1 {
                        tensor[[0, 0, i, j]] = ((now / 255f32) - 0.456f32) / 0.224f32;
                    } else {
                        let r = image[[0, i, j]] as f32;
                        let g = image[[1, i, j]] as f32;
                        let b = image[[2, i, j]] as f32;
                        tensor[[0, 0, i, j]] = ((r / 255f32) - 0.485f32) / 0.229f32;
                        tensor[[0, 1, i, j]] = ((g / 255f32) - 0.456f32) / 0.224f32;
                        tensor[[0, 2, i, j]] = ((b / 255f32) - 0.406f32) / 0.225f32;
                    }
                } else {
                    tensor[[0, 0, i, j]] = ((now / 255f32) - 0.5f32) / 0.5f32;
                }
            }
        }

        if word {
            Ok((&self.session.run(ort::inputs![tensor]?)?[1])
                .try_extract_tensor::<i64>()?
                .iter()
                .map(|&v| charset[v as usize].to_string())
                .collect::<String>())
        } else {
            if self.diy {
                // todo: 自定义模型未经测试
                let result = &self.session.run(ort::inputs![tensor]?)?[0];

                let result = result.try_extract_tensor::<u32>()?;

                let mut last_item = 0;

                Ok(result
                    .iter()
                    .filter(|&&v| {
                        if v != 0 && v != last_item {
                            last_item = v;
                            true
                        } else {
                            false
                        }
                    })
                    .map(|&v| charset[v as usize].to_string())
                    .collect::<String>())
            } else {
                let result = &self.session.run(ort::inputs![tensor]?)?[0];

                let result = result.try_extract_tensor::<f32>()?;

                let mut last_item = 0;

                // 输入长这样 [[[1,2,3,4], [1,2,3,4], [1,2,3,4]]]
                // 我们要获取   ^^^^^^^^^  ^^^^^^^^^  ^^^^^^^^^
                // 最后结果 [3, 3, 3]
                // 这是最大值的索引
                let result = result
                    .rows()
                    .into_iter()
                    .map(|v| {
                        // 找出数组中元素值最大的那个，然后获取他在数组中的索引
                        v.iter()
                            .enumerate()
                            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                            .unwrap_or((0, &0.0))
                            .0
                    })
                    .collect::<Vec<usize>>();

                // 过滤无效字符
                Ok(result
                    .iter()
                    .filter(|&&v| {
                        if v != 0 && v != last_item {
                            last_item = v;
                            true
                        } else {
                            false
                        }
                    })
                    .map(|&v| charset[v as usize].to_string())
                    .collect::<String>())
            }
        }
    }

    /// 根据坐标裁剪图片，然后进行内容识别。
    pub fn classification_bbox<I>(
        &self,
        image: I,
        bbox: &Vec<BBox>,
    ) -> anyhow::Result<Vec<(BBox, String)>>
    where
        I: AsRef<[u8]>,
    {
        let image = image::load_from_memory(image.as_ref())?;

        let mut result = Vec::new();

        for i in bbox {
            let mut buffer = std::io::Cursor::new(Vec::new());

            // todo: 使用 png 格式会不会有问题啊？
            image::imageops::crop_imm(&image, i.x1, i.y1, i.x2 - i.x1 + 1, i.y2 - i.y1 + 1)
                .to_image()
                .write_to(&mut buffer, image::ImageFormat::Png)?;

            result.push((i.clone(), self.classification(buffer.into_inner())?));
        }

        Ok(result)
    }

    /// 根据坐标裁剪图片，然后进行内容识别。
    pub fn classification_bbox_with_path<P>(
        &self,
        path: P,
        bbox: &Vec<BBox>,
    ) -> anyhow::Result<Vec<(BBox, String)>>
    where
        P: AsRef<std::path::Path>,
    {
        self.classification_bbox(std::fs::read(path)?, bbox)
    }

    /// 目标检测。
    pub fn detection<I>(&self, image: I) -> anyhow::Result<Vec<BBox>>
    where
        I: AsRef<[u8]>,
    {
        #[derive(Debug, Clone, Copy)]
        struct ScoresBBox {
            scores: f32,
            x1: f32,
            y1: f32,
            x2: f32,
            y2: f32,
        }

        let original_image = image::load_from_memory(image.as_ref())?;

        // 图片缩放到模型大小
        let x = MODEL_WIDTH as f32 / original_image.width() as f32;
        let y = MODEL_HEIGHT as f32 / original_image.height() as f32;
        let ratio = x.min(y);
        let width = (original_image.width() as f32 * ratio) as u32;
        let hight = (original_image.height() as f32 * ratio) as u32;

        // todo: 要不要使用 resize_exact？
        let image = original_image
            .resize(width, hight, image::imageops::FilterType::Triangle)
            .to_rgb8();

        // 空白部分使用灰色填充
        let image = image::RgbImage::from_fn(MODEL_WIDTH, MODEL_HEIGHT, |x, y| {
            *image
                .get_pixel_checked(x, y)
                .unwrap_or(&image::Rgb([114, 114, 114]))
        });

        // 图片转换到张量
        let w = MODEL_WIDTH as usize;
        let h = MODEL_HEIGHT as usize;
        let mut input_tensor = ndarray::Array::from_shape_vec((1, 3, h, w), vec![0f32; 3 * h * w])?;

        for i in 0..image.width() {
            for j in 0..image.height() {
                // 为什么这里他妈的 x 和 y 是相反的？
                let now = image[(j, i)];

                input_tensor[[0, 0, i as usize, j as usize]] = now[0] as f32;
                input_tensor[[0, 1, i as usize, j as usize]] = now[1] as f32;
                input_tensor[[0, 2, i as usize, j as usize]] = now[2] as f32;
            }
        }

        // 首先将原始图像的宽度和高度与模型的宽度和高度进行比较，得到一个缩放比例 gain
        // 然后对每个物体进行检测，如果其得分小于 SCORE_THR，则跳过该物体
        // 接着，对物体的坐标进行调整，最后将调整后的坐标加入到结果列表中
        // 最后，对结果列表中的物体进行非极大值抑制 (NMS) 处理，得到最终的检测结果
        let output_tensor = &self.session.run(ort::inputs![input_tensor]?)?[0];
        let output_tensor = output_tensor.try_extract_tensor::<f32>()?;
        let x = MODEL_WIDTH as f32 / original_image.width() as f32;
        let y = MODEL_HEIGHT as f32 / original_image.height() as f32;
        let gain = x.min(y);
        let mut result = Vec::new();

        for i in 0..output_tensor.len() / 6 {
            let scores = output_tensor[[0, i, 4]] * output_tensor[[0, i, 5]];

            if scores < SCORE_THR {
                continue;
            }

            let mut x1 = output_tensor[[0, i, 0]];
            let mut y1 = output_tensor[[0, i, 1]];
            let mut x2 = output_tensor[[0, i, 2]];
            let mut y2 = output_tensor[[0, i, 3]];

            x1 = (x1 + GRIDS[i * 2] as f32) * EXPANDED_STRIDES[i] as f32;
            y1 = (y1 + GRIDS[i * 2 + 1] as f32) * EXPANDED_STRIDES[i] as f32;
            x2 = x2.exp() * EXPANDED_STRIDES[i] as f32;
            y2 = y2.exp() * EXPANDED_STRIDES[i] as f32;

            result.push(ScoresBBox {
                scores,
                x1: (x1 - x2 / 2f32) / gain,
                y1: (y1 - y2 / 2f32) / gain,
                x2: (x1 + x2 / 2f32) / gain,
                y2: (y1 + y2 / 2f32) / gain,
            });
        }

        // 在目标检测中，非极大值抑制 (NMS) 用于去除冗余的边界框
        // 首先，NMS 将所有边界框按照得分从高到低排序
        // 然后选择得分最高的边界框，并将与其 (交并比) 大于一定阈值的边界框从列表中删除
        // 接着，重复这个过程，直到所有的边界框都被处理完毕
        // 因此，NMS 的过程是从得分最高的边界框开始，逐渐筛选出最优的边界框
        let mut scores = Vec::new();
        let mut areas = Vec::new();

        for i in &result {
            scores.push(i.scores);
            areas.push((i.x2 - i.x1 + 1f32) * (i.y2 - i.y1 + 1f32));
        }

        let mut array = scores;
        let mut order = (0..array.len()).collect::<Vec<_>>();

        for i in 0..array.len() {
            for j in i + 1..array.len() {
                array.swap(i, j);
                order.swap(i, j);
            }
        }

        let mut keep = Vec::new();

        while !order.is_empty() {
            let i = order[0];

            keep.push(i);

            let mut new_order = Vec::new();

            for j in 1..order.len() {
                let temp = result[order[j]];
                let xx1 = result[i].x1.max(temp.x1);
                let yy1 = result[i].y1.max(temp.y1);
                let xx2 = result[i].x2.min(temp.x2);
                let yy2 = result[i].y2.min(temp.y2);
                let ww = 0f32.max(xx2 - xx1 + 1f32);
                let hh = 0f32.max(yy2 - yy1 + 1f32);
                let inter = ww * hh;
                let ovr = inter / (areas[j] + areas[order[j]] - inter);

                if ovr <= NMS_THR {
                    new_order.push(order[j]);
                }
            }

            order = new_order;
        }

        let mut new_result = Vec::new();

        for i in keep {
            let mut point = result[i];

            if point.x1 < 0f32 {
                point.x1 = 0f32;
            } else if point.x1 > original_image.width() as f32 {
                point.x1 = (original_image.width() - 1) as f32;
            }

            if point.y1 < 0f32 {
                point.y1 = 0f32;
            } else if point.y1 > original_image.height() as f32 {
                point.y1 = (original_image.height() - 1) as f32;
            }

            if point.x2 < 0f32 {
                point.x2 = 0f32;
            } else if point.x2 > original_image.width() as f32 {
                point.x2 = (original_image.width() - 1) as f32;
            }

            if point.y2 < 0f32 {
                point.y2 = 0f32;
            } else if point.y2 > original_image.height() as f32 {
                point.y2 = (original_image.height() - 1) as f32;
            }

            new_result.push(crate::BBox {
                x1: point.x1 as u32,
                y1: point.y1 as u32,
                x2: point.x2 as u32,
                y2: point.y2 as u32,
            });
        }

        Ok(new_result)
    }

    /// 目标检测。
    pub fn detection_with_path<P>(&self, path: P) -> anyhow::Result<Vec<BBox>>
    where
        P: AsRef<std::path::Path>,
    {
        self.detection(std::fs::read(path)?)
    }
}

#[cfg(test)]
mod tests {
    use image::Pixel;

    use super::*;

    #[test]
    fn classification_probability() {
        let mut ddddocr = ddddocr_classification().unwrap();

        // CharsetRange::LowercaseUppercase 大写字母和小写字母
        ddddocr.set_ranges(3);

        let mut result = ddddocr
            .classification_probability(include_bytes!("../image/3.png"))
            .unwrap();

        println!("识别结果: {}", result.get_text());
        println!("识别可信度: {}", result.get_confidence());

        // 哦呀，看来数据有点儿太多了，小心卡死哦！
        println!("概率: {}", result.json());
    }

    #[test]
    fn classification_filter() {
        let ddddocr = ddddocr_classification().unwrap();

        println!(
            "{}",
            ddddocr
                .classification_with_filter(include_bytes!("../image/4.png"), "green")
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification_with_filter(
                    include_bytes!("../image/4.png"),
                    [((40, 50, 50), (80, 255, 255))]
                )
                .unwrap()
        );
    }

    #[test]
    fn classification() {
        let ddddocr = ddddocr_classification().unwrap();

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/1.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/2.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/3.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/4.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/su.png"))
                .unwrap()
        );
    }

    #[test]
    fn classification_old() {
        let ddddocr = ddddocr_classification_old().unwrap();

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/1.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/2.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/3.png"))
                .unwrap()
        );

        println!(
            "{}",
            ddddocr
                .classification(include_bytes!("../image/4.png"))
                .unwrap()
        );
    }

    #[test]
    fn classification_bbox() {
        let input = include_bytes!("../image/6.jpg");
        let ddddocr = ddddocr_detection().unwrap();
        let result = ddddocr.detection(input).unwrap();
        let ddddocr = ddddocr_classification().unwrap();
        let result = ddddocr.classification_bbox(input, &result).unwrap();

        println!("{:?}", result);
    }

    #[test]
    fn detection() {
        let ddddocr = ddddocr_detection().unwrap();
        let input = include_bytes!("../image/5.jpg");
        let result = ddddocr.detection(input).unwrap();

        println!("{:?}", result);

        // 绘制红框
        let mut image = image::load_from_memory(input).unwrap().to_rgb8();

        for v in result {
            for i in v.x1 as u32..=v.x2 as u32 {
                image[(i, v.y1 as u32)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.x1 as u32..=v.x2 as u32 {
                image[(i, v.y2 as u32)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.y1 as u32..=v.y2 as u32 {
                image[(v.x1 as u32, i)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.y1 as u32..=v.y2 as u32 {
                image[(v.x2 as u32, i)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }
        }

        image.save("./output1.jpg").unwrap();

        let input = include_bytes!("../image/6.jpg");
        let result = ddddocr.detection(input).unwrap();

        println!("{:?}", result);

        // 绘制红框
        let mut image = image::load_from_memory(input).unwrap().to_rgb8();

        for v in result {
            for i in v.x1 as u32..=v.x2 as u32 {
                image[(i, v.y1 as u32)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.x1 as u32..=v.x2 as u32 {
                image[(i, v.y2 as u32)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.y1 as u32..=v.y2 as u32 {
                image[(v.x1 as u32, i)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }

            for i in v.y1 as u32..=v.y2 as u32 {
                image[(v.x2 as u32, i)] = *image::Rgb::from_slice(&[237, 28, 36]);
            }
        }

        image.save("./output2.jpg").unwrap();
    }

    #[test]
    fn slide_match() {
        let result = crate::slide_match(
            include_bytes!("../image/hk.png"),
            include_bytes!("../image/bg.png"),
        )
        .unwrap();

        println!("{:?}", result);

        let result = crate::slide_match(
            include_bytes!("../image/a.png"),
            include_bytes!("../image/b.png"),
        )
        .unwrap();

        println!("{:?}", result);

        let result = crate::simple_slide_match(
            include_bytes!("../image/a.png"),
            include_bytes!("../image/b.png"),
        )
        .unwrap();

        println!("{:?}", result);
    }

    #[test]
    fn comparison_match() {
        let result = crate::slide_comparison(
            include_bytes!("../image/c.jpg"),
            include_bytes!("../image/d.jpg"),
        )
        .unwrap();
        println!("{:?}", result);
    }
}
