/// 初始化内容识别。
pub fn ddddocr_classification() -> anyhow::Result<Ddddocr<'static>> {
    let charset = include_str!("../model/common.json");
    Ddddocr::new(
        include_bytes!("../model/common.onnx"),
        serde_json::from_str(charset).unwrap(),
    )
}

/// 使用旧模型初始化内容识别。
pub fn ddddocr_classification_old() -> anyhow::Result<Ddddocr<'static>> {
    let charset = include_str!("../model/common_old.json");
    Ddddocr::new(
        include_bytes!("../model/common_old.onnx"),
        serde_json::from_str(charset).unwrap(),
    )
}

/// 初始化目标检测。
pub fn ddddocr_detection() -> anyhow::Result<Ddddocr<'static>> {
    Ddddocr::new_model(include_bytes!("../model/common_det.onnx"))
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
        "背景图片的宽度必须大于等于目标图标的宽度"
    );
    anyhow::ensure!(
        background_image.height() >= target_image.height(),
        "背景图片的高度必须大于等于目标图标的高度"
    );

    // 裁剪图片
    let image = target_image.to_rgba8();

    let mut target_x = 0;
    let mut target_y = 0;
    let mut end_x = 0;
    let mut end_y = 0;
    for x in 0..image.width() {
        for y in 0..image.height() {
            let p = image[(x, y)];
            if p[3] == 0 {
                if target_y != 0 && end_y == 0 {
                    end_y = y;
                }
                if target_x != 0 && end_x == 0 {
                    end_x = x;
                }
            } else if target_y == 0 {
                target_y = y;
                end_y = 0;
            } else if y < target_y {
                target_y = y;
                end_y = 0;
            }
        }
        if target_x == 0 && target_y != 0 {
            target_x = x;
        }
        if end_y != 0 {
            end_x = x;
        }
    }

    // 图片转换到灰度图
    let target_image = image::imageops::grayscale(
        &image::imageops::crop_imm(
            &image,
            target_x,
            target_y,
            end_x - target_x,
            end_y - target_y,
        )
        .to_image(),
    );

    // 使用 canny 进行边缘检测。然后对背景图片进行同样的处理
    // 接着，使用 match_template 函数进行模板匹配，得到匹配结果矩阵
    // 然后使用 find_extremes 函数找到结果矩阵中的最大值和最小值
    // 并得到最大值所在的位置 loc，根据目标图片的大小和 loc 计算出目标物体的位置信息
    let target_image = imageproc::edges::canny(&target_image, 100.0, 200.0);
    let background_image = imageproc::edges::canny(&background_image.to_luma8(), 100.0, 200.0);
    let result = imageproc::template_matching::match_template(
        &background_image,
        &target_image,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    );
    let result = imageproc::template_matching::find_extremes(&result);
    Ok(SlideBBox {
        target_y,
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

/// 如果小图无过多背景部分，
/// 可以使用简单滑块匹配。
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
    let result = imageproc::template_matching::match_template(
        &background_image,
        &target_image,
        imageproc::template_matching::MatchTemplateMethod::CrossCorrelationNormalized,
    );
    let result = imageproc::template_matching::find_extremes(&result);
    Ok(SlideBBox {
        target_y: 0,
        x1: result.max_value_location.0,
        y1: result.max_value_location.1,
        x2: result.max_value_location.0 + target_image.width(),
        y2: result.max_value_location.1 + target_image.height(),
    })
}

/// 如果小图无过多背景部分，
/// 可以使用简单滑块匹配。
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
    let mut start_y = 0;
    let mut start_x = 0;
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
    sha256 != "90e432635bfb100ac6b856220a10b186053264cc307dd049d58e8ea2643cb6e9"
        && sha256 != "b1d5e1344954355b10b40632062b8a9def06bcc33229fb667274662bf31dcf3f"
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
    pub target_y: u32,
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

pub trait MapJson {
    fn json(&self) -> String;
}

impl MapJson for BBox {
    fn json(&self) -> String {
        unsafe { serde_json::to_string(self).unwrap_unchecked() }
    }
}

impl MapJson for Vec<BBox> {
    fn json(&self) -> String {
        unsafe { serde_json::to_string(self).unwrap_unchecked() }
    }
}

impl MapJson for SlideBBox {
    fn json(&self) -> String {
        unsafe { serde_json::to_string(self).unwrap_unchecked() }
    }
}

impl MapJson for (u32, u32) {
    fn json(&self) -> String {
        unsafe { serde_json::to_string(self).unwrap_unchecked() }
    }
}

lazy_static::lazy_static! {
    static ref ENVIRONMENT: onnxruntime::environment::Environment =
        onnxruntime::environment::Environment::builder()
            .build()
            .expect("environment initialization exception");
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

#[derive(Debug)]
pub struct Ddddocr<'a> {
    diy: bool,
    session: onnxruntime::session::Session<'a>,
    charset: Option<std::borrow::Cow<'a, Charset>>,
}

/// 我也不知道这里是不是安全的，但我多线程测试过，没有发现异常。
unsafe impl<'a> Send for Ddddocr<'a> {}

/// 我也不知道这里是不是安全的，但我多线程测试过，没有发现异常。
unsafe impl<'a> Sync for Ddddocr<'a> {}

/// 因为自带模型和自定义模型的参数不同，
/// 所有在创建模型的时候会自动判断是否为自定义模型。
impl<'a> Ddddocr<'a> {
    /// 从内存加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
    pub fn new<MODEL>(model: MODEL, charset: Charset) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .with_model_from_memory(model)?,
            charset: Some(std::borrow::Cow::Owned(charset)),
        })
    }

    /// 从内存加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
    pub fn new_ref<MODEL>(model: MODEL, charset: &'a Charset) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .with_model_from_memory(model)?,
            charset: Some(std::borrow::Cow::Borrowed(charset)),
        })
    }

    /// 从内存加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_cuda<MODEL>(model: MODEL, charset: Charset, device_id: i32) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: Self::is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .use_cuda(device_id)?
                .with_model_from_memory(model)?,
            charset: Some(charset),
        })
    }

    /// 从内存加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_cuda_ref<MODEL>(
        model: MODEL,
        charset: &'a Charset,
        device_id: i32,
    ) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: Self::is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .use_cuda(device_id)?
                .with_model_from_memory(model)?,
            charset: Some(std::borrow::Cow::Borrowed(charset)),
        })
    }

    /// 从内存加载模型，
    /// 只能使用目标检测，
    /// 使用内容识别会恐慌。
    pub fn new_model<MODEL>(model: MODEL) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .with_model_from_memory(model)?,
            charset: None,
        })
    }

    /// 从内存加载模型，
    /// 只能使用目标检测，
    /// 使用内容识别会恐慌。
    #[cfg(feature = "cuda")]
    pub fn new_model_cuda<MODEL>(model: MODEL, device_id: i32) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            diy: Self::is_diy(model.as_ref()),
            session: ENVIRONMENT
                .new_session_builder()?
                .use_cuda(device_id)?
                .with_model_from_memory(model)?,
            charset: None,
        })
    }

    /// 从文件加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
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

    /// 从文件加载模型和字符集，
    /// 只能使用内容识别，
    /// 使用目标检测会恐慌。
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

    /// 从文件加载模型，
    /// 只能使用目标检测，
    /// 使用内容识别会恐慌。
    pub fn with_model<P>(model: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Self::new_model(std::fs::read(model)?)
    }

    /// 从文件加载模型，
    /// 只能使用目标检测，
    /// 使用内容识别会恐慌。
    #[cfg(feature = "cuda")]
    pub fn with_model_cuda<P>(model: P, device_id: i32) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Self::new_model_cuda(std::fs::read(model)?, device_id)
    }

    /// 内容识别。
    pub fn classification<I>(&mut self, image: I) -> anyhow::Result<String>
    where
        I: AsRef<[u8]>,
    {
        let image = image::load_from_memory(image.as_ref())?;
        let charset = self.charset.as_ref().unwrap();
        let word = charset.word;
        let resize = charset.image;
        let channel = charset.channel;
        let charset = &charset.charset;

        // 使用 ANTIALIAS (Lanczos3) 缩放图片
        let image = if resize[0] == -1 {
            if word {
                image.resize(
                    resize[1] as u32,
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                image.resize(
                    image.width() * resize[1] as u32 / image.height(),
                    resize[1] as u32,
                    image::imageops::FilterType::Lanczos3,
                )
            }
        } else {
            image.resize(
                resize[0] as u32,
                resize[1] as u32,
                image::imageops::FilterType::Lanczos3,
            )
        };

        // 设置图片的通道数为模型所需的通道数
        let image_bytes = if channel == 1 {
            image::EncodableLayout::as_bytes(image.to_luma8().as_ref()).to_vec()
        } else {
            image.to_rgb8().to_vec()
        };

        // 图片转换到张量
        let channel = channel as usize;
        let width = image.width() as usize;
        let height = image.height() as usize;
        let image =
            onnxruntime::ndarray::Array::from_shape_vec((channel, height, width), image_bytes)?;
        let mut tensor = onnxruntime::ndarray::Array::from_shape_vec(
            (1, channel, height, width),
            vec![0f32; height * width],
        )?;
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
            Ok(self.session.run::<_, i64, _>(vec![tensor])?[1]
                .iter()
                .map(|&v| charset[v as usize].to_string())
                .collect::<String>())
        } else {
            // 过滤无效字符
            let mut last_item = 0;
            Ok(self.session.run::<_, i64, _>(vec![tensor])?[0]
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

    /// 内容识别。
    pub fn classification_with_path<P>(&mut self, path: P) -> anyhow::Result<String>
    where
        P: AsRef<std::path::Path>,
    {
        self.classification(std::fs::read(path)?)
    }

    /// 目标检测。
    pub fn detection<I>(&mut self, image: I) -> anyhow::Result<Vec<BBox>>
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
        let mut input_tensor =
            onnxruntime::ndarray::Array::from_shape_vec((1, 3, h, w), vec![0f32; 3 * h * w])?;
        for i in 0..image.width() {
            for j in 0..image.height() {
                // 为什么这里的 x 和 y 是相反的？
                // 因为傻狗 opencv 中的 Mat::at 函数就是这么设计的
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
        let output_tensor = &self.session.run::<_, f32, _>(vec![input_tensor])?[0];
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
    pub fn detection_with_path<P>(&mut self, path: P) -> anyhow::Result<Vec<BBox>>
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
    fn classification() {
        let mut ddddocr = ddddocr_classification().unwrap();
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
    fn classification_old() {
        let mut ddddocr = ddddocr_classification_old().unwrap();
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
    fn detection() {
        let mut ddddocr = ddddocr_detection().unwrap();
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
