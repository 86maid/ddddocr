use anyhow::Ok;

lazy_static::lazy_static! {
    static ref ENVIRONMENT: onnxruntime::environment::Environment = onnxruntime::environment::Environment::builder().build().expect("environment initialization exception");
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
                tensor[(0, 0, i, j)] = ((now / 255f32) - 0.5f32) / 0.5f32;
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

    /// 目标检测，
    /// 每个字对应 (x1, y1, x2, y2)。
    pub fn detection(&mut self, image: &[u8]) -> anyhow::Result<Vec<(u32, u32, u32, u32)>> {
        const MODEL_WIDTH: u32 = 416;
        const MODEL_HEIGHT: u32 = 416;
        const STRIDES: [u32; 3] = [8, 16, 32];
        let image = image::load_from_memory(image)?;
        let ratio = (MODEL_WIDTH as f32 / image.width() as f32)
            .min(MODEL_HEIGHT as f32 / image.height() as f32);
        let rgb_image = image
            .resize(
                (image.width() as f32 * ratio) as u32,
                (image.height() as f32 * ratio) as u32,
                image::imageops::FilterType::Triangle,
            )
            .to_rgb8();
        let gray_background_image =
            image::ImageBuffer::from_fn(MODEL_WIDTH, MODEL_HEIGHT, |x, y| {
                *rgb_image
                    .get_pixel_checked(x, y)
                    .unwrap_or(&image::Rgb([114, 114, 114]))
            });
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
        let mut tensor = onnxruntime::ndarray::Array::from_shape_vec(
            (1, 3, MODEL_HEIGHT as usize, MODEL_WIDTH as usize),
            vec![0f32; 3 * MODEL_HEIGHT as usize * MODEL_WIDTH as usize],
        )?;
        for i in 0..gray_background_image.height() {
            for j in 0..gray_background_image.width() {
                let now = gray_background_image[(i, j)];
                tensor[(0, 0, i as usize, j as usize)] = now[0] as f32;
                tensor[(0, 1, i as usize, j as usize)] = now[1] as f32;
                tensor[(0, 2, i as usize, j as usize)] = now[2] as f32;
            }
        }
        let mut result = Vec::<(u32, u32, u32, u32)>::new();
        let output = self.session.run::<_, f32, _>(vec![tensor])?[0]
            .iter()
            .map(|v| *v)
            .collect::<Vec<_>>();

        let hsizes = STRIDES.map(|v| MODEL_HEIGHT / v).to_vec();
        let wsizes = STRIDES.map(|v| MODEL_WIDTH / v).to_vec();

        let mut grids = Vec::new();
        let mut expanded_strides = Vec::new();

        for ((hsize, wsize), stride) in hsizes.into_iter().zip(wsizes).zip(STRIDES) {
            fn meshgrid(x: &[u32], y: &[u32]) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
                let mut xx = vec![vec![0; x.len()]; y.len()];
                let mut yy = vec![vec![0; x.len()]; y.len()];
                for i in 0..y.len() {
                    for j in 0..x.len() {
                        xx[i][j] = x[j];
                        yy[i][j] = y[i];
                    }
                }
                (xx, yy)
            }

            let (xv, yv) = meshgrid(
                &(0..wsize)
                    .enumerate()
                    .map(|v| v.0 as u32)
                    .collect::<Vec<u32>>(),
                &(0..hsize)
                    .enumerate()
                    .map(|v| v.0 as u32)
                    .collect::<Vec<u32>>(),
            );

            fn stack(x: Vec<Vec<u32>>, y: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
                let mut result = vec![vec![0; x.len()]; x.len() * y.len()];
                x.into_iter().zip(y).for_each(|(a, b)| {
                    result.push(a);
                    result.push(b);
                });
                result
            }

            let size = xv.len();
            let grid = stack(xv, yv);
            grids.push(grid);
            expanded_strides.push(vec![vec![stride; size]; size * size])
        }

        todo!("没写完...");

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
        println!("{:?}", ddddocr.detection(include_bytes!("../image/5.jpg")));
        println!("{:?}", ddddocr.detection(include_bytes!("../image/6.jpg")));
    }
}
