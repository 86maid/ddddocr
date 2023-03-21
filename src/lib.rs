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

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

pub struct Ddddocr<'a> {
    charset: Vec<String>,
    session: onnxruntime::session::Session<'a>,
}

impl<'a> Ddddocr<'a> {
    /// 从内存加载模型和字符集
    pub fn new<MODEL>(model: MODEL, charset: Vec<String>) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            charset,
            session: ENVIRONMENT
                .new_session_builder()?
                .with_model_from_memory(model)?,
        })
    }

    /// 从内存加载模型，
    /// 只能使用目标检测。
    pub fn new_model<MODEL>(model: MODEL) -> anyhow::Result<Self>
    where
        MODEL: AsRef<[u8]>,
    {
        Ok(Self {
            charset: Vec::new(),
            session: ENVIRONMENT
                .new_session_builder()?
                .with_model_from_memory(model)?,
        })
    }

    /// 从文件加载模型和字符集
    pub fn with_model_charset<PATH1, PATH2>(model: PATH1, charset: PATH2) -> anyhow::Result<Self>
    where
        PATH1: AsRef<std::path::Path>,
        PATH2: AsRef<std::path::Path>,
    {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Charset {
            charset: Vec<String>,
            word: bool,
            image: [i32; 2],
            channel: i32,
        }
        Self::new(
            std::fs::read(model)?,
            serde_json::from_str::<Charset>(&std::fs::read_to_string(charset)?)?.charset,
        )
    }

    /// 从文件加载模型，
    /// 只能使用目标检测。
    pub fn with_model<PATH>(model: PATH) -> anyhow::Result<Self>
    where
        PATH: AsRef<std::path::Path>,
    {
        Self::new_model(std::fs::read(model)?)
    }

    /// 内容识别
    pub fn classification(&mut self, image: &[u8]) -> anyhow::Result<String> {
        let image = image::load_from_memory(image)?;
        let image = image::imageops::grayscale(&image.resize(
            image.width() * 64 / image.height(),
            64,
            image::imageops::FilterType::Triangle,
        ));
        self.session.inputs = vec![onnxruntime::session::Input {
            name: "input1".to_string(),
            input_type: onnxruntime::TensorElementDataType::Float,
            dimensions: vec![
                Some(1),
                Some(1),
                Some(64),
                Some(image.width() * 64 / image.height()),
            ],
        }];
        self.session.outputs = vec![onnxruntime::session::Output {
            name: "output".to_string(),
            output_type: onnxruntime::TensorElementDataType::Int64,
            dimensions: vec![None],
        }];
        let width = image.width() as usize;
        let height = image.height() as usize;
        let image = onnxruntime::ndarray::Array::from_shape_vec((height, width), image.into_vec())?;
        let mut tensor = onnxruntime::ndarray::Array::from_shape_vec(
            (1, 1, height, width),
            vec![0f32; height * width],
        )?;
        for i in 0..height {
            for j in 0..width {
                let now = image[(i, j)] as f32;
                tensor[[0, 0, i, j]] = ((now / 255f32) - 0.5f32) / 0.5f32;
            }
        }
        Ok(self.session.run::<_, i64, _>(vec![tensor])?[0]
            .iter()
            .map(|&v| self.charset[v as usize].to_string())
            .collect::<String>())
    }

    /// 内容识别
    pub fn classification_with_path<PATH>(&mut self, path: PATH) -> anyhow::Result<String>
    where
        PATH: AsRef<std::path::Path>,
    {
        self.classification(&std::fs::read(path)?)
    }

    /// 目标检测
    pub fn detection(&mut self, image: &[u8]) -> anyhow::Result<Vec<Point>> {
        // 将图片缩放到模型大小
        fn resize(image: &image::DynamicImage) -> image::RgbImage {
            let x = MODEL_WIDTH as f32 / image.width() as f32;
            let y = MODEL_HEIGHT as f32 / image.height() as f32;
            let ratio = x.min(y);
            let width = (image.width() as f32 * ratio) as u32;
            let hight = (image.height() as f32 * ratio) as u32;
            let image = image
                .resize(width, hight, image::imageops::FilterType::Triangle)
                .to_rgb8();
            image::RgbImage::from_fn(MODEL_WIDTH, MODEL_HEIGHT, |x, y| {
                *image
                    .get_pixel_checked(x, y)
                    .unwrap_or(&image::Rgb([114, 114, 114]))
            })
        }

        // 将图片转换到张量
        fn tensor(
            image: image::RgbImage,
        ) -> anyhow::Result<onnxruntime::ndarray::Array<f32, onnxruntime::ndarray::Dim<[usize; 4]>>>
        {
            let w = MODEL_WIDTH as usize;
            let h = MODEL_HEIGHT as usize;
            let mut result = onnxruntime::ndarray::Array::from_shape_vec(
                (1, 3, h, w),
                vec![0f32; 1 * 3 * h * w],
            )?;
            for i in 0..image.width() {
                // 为什么要反向迭代？
                // 别发神经乱改
                for j in (0..image.height()).rev() {
                    let now = image.get_pixel(j, i);
                    result[(0, 0, i as usize, j as usize)] = now[0] as f32;
                    result[(0, 1, i as usize, j as usize)] = now[1] as f32;
                    result[(0, 2, i as usize, j as usize)] = now[2] as f32;
                }
            }
            Ok(result)
        }

        // 解析推理结果张量
        fn parse(
            tensor: &onnxruntime::tensor::OrtOwnedTensor<
                f32,
                onnxruntime::ndarray::Dim<onnxruntime::ndarray::IxDynImpl>,
            >,
            original_image: image::DynamicImage,
        ) -> Vec<Point> {
            #[derive(Debug, Clone, Copy)]
            struct Point {
                scores: f32,
                x1: f32,
                y1: f32,
                x2: f32,
                y2: f32,
            }

            fn sort(mut array: Vec<f32>) -> Vec<usize> {
                let mut result = (0..array.len()).into_iter().collect::<Vec<_>>();
                for i in 0..array.len() {
                    for j in i + 1..array.len() {
                        let temp = array[i];
                        array[i] = array[j];
                        array[j] = temp;
                        let temp = result[i];
                        result[i] = result[j];
                        result[j] = temp;
                    }
                }
                result
            }

            let x = MODEL_WIDTH as f32 / original_image.width() as f32;
            let y = MODEL_HEIGHT as f32 / original_image.height() as f32;
            let gain = x.min(y);
            let mut result = Vec::new();
            for i in 0..tensor.len() / 6 {
                let scores = tensor[[0, i, 4]] * tensor[[0, i, 5]];
                if scores < SCORE_THR {
                    continue;
                }
                let mut x1 = tensor[[0, i, 0]];
                let mut y1 = tensor[[0, i, 1]];
                let mut x2 = tensor[[0, i, 2]];
                let mut y2 = tensor[[0, i, 3]];
                x1 = (x1 as f32 + GRIDS[i * 2 + 0] as f32) * EXPANDED_STRIDES[i] as f32;
                y1 = (y1 as f32 + GRIDS[i * 2 + 1] as f32) * EXPANDED_STRIDES[i] as f32;
                x2 = x2.exp() as f32 * EXPANDED_STRIDES[i] as f32;
                y2 = y2.exp() as f32 * EXPANDED_STRIDES[i] as f32;
                result.push(Point {
                    scores,
                    x1: (x1 - x2 / 2f32) / gain,
                    y1: (y1 - y2 / 2f32) / gain,
                    x2: (x1 + x2 / 2f32) / gain,
                    y2: (y1 + y2 / 2f32) / gain,
                });
            }
            let mut scores = Vec::new();
            let mut areas = Vec::new();
            for i in &result {
                scores.push(i.scores);
                areas.push((i.x2 - i.x1 + 1f32) * (i.y2 - i.y1 + 1f32));
            }
            let mut order = sort(scores);
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
                new_result.push(crate::Point {
                    x1: point.x1 as u32,
                    y1: point.y1 as u32,
                    x2: point.x2 as u32,
                    y2: point.y2 as u32,
                });
            }
            new_result
        }

        let original_image = image::load_from_memory(image)?;
        let image = resize(&original_image);
        let tensor = tensor(image)?;
        self.session.inputs = vec![onnxruntime::session::Input {
            name: "images".to_string(),
            input_type: onnxruntime::TensorElementDataType::Float,
            dimensions: vec![Some(1), Some(3), Some(MODEL_HEIGHT), Some(MODEL_WIDTH)],
        }];
        self.session.outputs = vec![onnxruntime::session::Output {
            name: "output".to_string(),
            output_type: onnxruntime::TensorElementDataType::Float,
            dimensions: vec![None],
        }];
        let tensor = &self.session.run::<_, f32, _>(vec![tensor])?[0];
        let result = parse(tensor, original_image);
        Ok(result)
    }
}

pub fn ddddocr_classification<'a>() -> anyhow::Result<Ddddocr<'a>> {
    let charset = include!("../model/common_text.txt");
    Ddddocr::new(
        include_bytes!("../model/common.onnx"),
        charset.iter().map(|v| v.to_string()).collect(),
    )
}

pub fn ddddocr_classification_old<'a>() -> anyhow::Result<Ddddocr<'a>> {
    let charset = include!("../model/common_old_text.txt");
    Ddddocr::new(
        include_bytes!("../model/common_old.onnx"),
        charset.iter().map(|v| v.to_string()).collect(),
    )
}

pub fn ddddocr_detection<'a>() -> anyhow::Result<Ddddocr<'a>> {
    Ddddocr::new_model(include_bytes!("../model/common_det.onnx"))
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
        let input = include_bytes!("../image/6.jpg");
        let result = ddddocr.detection(input).unwrap();
        println!("{:?}", result);

        // 红框
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

        image.save("./output.jpg").unwrap();
    }
}
