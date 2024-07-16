#[no_mangle]
pub unsafe extern "stdcall" fn with_model_charset(
    model: *const u8,
    model_len: usize,
    charset: *const u8,
    charset_len: usize,
) -> *mut ddddocr::Ddddocr<'static> {
    let model = std::slice::from_raw_parts(model, model_len);
    let charset = std::slice::from_raw_parts(charset, charset_len);

    let charset = match serde_json::from_slice::<ddddocr::Charset>(&charset) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    match ddddocr::Ddddocr::new(model, charset) {
        Ok(this) => Box::into_raw(Box::new(this)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "stdcall" fn init_classification() -> *mut ddddocr::Ddddocr<'static> {
    match ddddocr::ddddocr_classification() {
        Ok(this) => Box::into_raw(Box::new(this)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "stdcall" fn init_classification_old() -> *mut ddddocr::Ddddocr<'static> {
    match ddddocr::ddddocr_classification_old() {
        Ok(this) => Box::into_raw(Box::new(this)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "stdcall" fn init_detection() -> *mut ddddocr::Ddddocr<'static> {
    match ddddocr::ddddocr_detection() {
        Ok(this) => Box::into_raw(Box::new(this)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn slide_match(
    target_image_ptr: *const u8,
    target_image_len: usize,
    background_image_ptr: *const u8,
    background_image_len: usize,
    result: *mut ddddocr::SlideBBox,
) -> i32 {
    let target_image = std::slice::from_raw_parts(target_image_ptr, target_image_len);
    let background_image = std::slice::from_raw_parts(background_image_ptr, background_image_len);

    match ddddocr::slide_match(target_image, background_image) {
        Ok(bbox) => {
            *result = bbox;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn simple_slide_match(
    target_image_ptr: *const u8,
    target_image_len: usize,
    background_image_ptr: *const u8,
    background_image_len: usize,
    result: *mut ddddocr::SlideBBox,
) -> i32 {
    let target_image = std::slice::from_raw_parts(target_image_ptr, target_image_len);
    let background_image = std::slice::from_raw_parts(background_image_ptr, background_image_len);

    match ddddocr::simple_slide_match(target_image, background_image) {
        Ok(bbox) => {
            *result = bbox;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn slide_comparison(
    target_image_ptr: *const u8,
    target_image_len: usize,
    background_image_ptr: *const u8,
    background_image_len: usize,
    result: *mut (u32, u32),
) -> i32 {
    let target_image = std::slice::from_raw_parts(target_image_ptr, target_image_len);
    let background_image = std::slice::from_raw_parts(background_image_ptr, background_image_len);

    match ddddocr::slide_comparison(target_image, background_image) {
        Ok(v) => {
            *result = v;
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn classification(
    this: *mut ddddocr::Ddddocr<'static>,
    image: *const u8,
    len: usize,
    png_fix: bool,
) -> *mut std::ffi::c_char {
    match (*this).classification(std::slice::from_raw_parts(image, len), png_fix) {
        Ok(result) => std::ffi::CString::new(result).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn classification_crop(
    this: *mut ddddocr::Ddddocr<'static>,
    image: *const u8,
    len: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> *mut std::ffi::c_char {
    let image = image::load_from_memory(std::slice::from_raw_parts(image, len));

    match image {
        Ok(v) => {
            let mut buffer = std::io::Cursor::new(Vec::new());

            if image::imageops::crop_imm(&v, x, y, width, height)
                .to_image()
                .write_to(&mut buffer, image::ImageFormat::Png)
                .is_err()
            {
                return std::ptr::null_mut();
            };

            let image = buffer.into_inner();

            classification(this, image.as_ptr(), image.len(), false)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn set_ranges(
    this: *mut ddddocr::Ddddocr<'static>,
    text: *const std::ffi::c_char,
) {
    (*this).set_ranges(std::ffi::CStr::from_ptr(text).to_string_lossy().to_string())
}

#[no_mangle]
pub unsafe extern "stdcall" fn classification_probability(
    this: *mut ddddocr::Ddddocr<'static>,
    image: *const u8,
    len: usize,
    png_fix: bool,
) -> *mut ddddocr::CharacterProbability {
    match (*this).classification_probability(std::slice::from_raw_parts(image, len), png_fix) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn character_probability_to_json(
    this: *mut ddddocr::CharacterProbability,
) -> *mut std::ffi::c_char {
    std::ffi::CString::new(ddddocr::MapJson::json(&*this))
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_character_probability_text(
    this: *mut ddddocr::CharacterProbability,
) -> *mut std::ffi::c_char {
    std::ffi::CString::new((*this).get_text())
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_ranges_len(this: *mut ddddocr::CharacterProbability) -> usize {
    (*this).charset.len()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_ranges_text(
    this: *mut ddddocr::CharacterProbability,
    join_char: u32,
) -> *mut std::ffi::c_char {
    std::ffi::CString::new(if join_char != 0 {
        ((*this).charset).join(&char::from_u32_unchecked(join_char).to_string())
    } else {
        ((*this).charset).join("")
    })
    .unwrap()
    .into_raw()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_ranges_char(
    this: *mut ddddocr::CharacterProbability,
    index: usize,
) -> u32 {
    ((&(*this).charset[index]).chars().next().unwrap_or('\0') as u32)
        .try_into()
        .unwrap()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_character_probability_len(
    this: *mut ddddocr::CharacterProbability,
    index: usize,
) -> usize {
    if index == usize::MAX {
        (*this).probability.len()
    } else {
        (*this).probability[index].len()
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_character_probability(
    this: *mut ddddocr::CharacterProbability,
    index1: usize,
    index2: usize,
) -> f32 {
    (*this).probability[index1][index2]
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_character_probability_copy(
    this: *mut ddddocr::CharacterProbability,
    index: usize,
    to: *mut f32,
) {
    std::ptr::copy_nonoverlapping(
        (*this).probability[index].as_ptr(),
        to,
        (&*this).probability[index].len(),
    );
}

#[no_mangle]
pub unsafe extern "stdcall" fn detection(
    this: *mut ddddocr::Ddddocr<'static>,
    image: *const u8,
    len: usize,
) -> *mut Vec<ddddocr::BBox> {
    match (*this).detection(std::slice::from_raw_parts(image, len)) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_bbox_vec_len(this: *mut Vec<ddddocr::BBox>) -> usize {
    (*this).len()
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_bbox_vec_index(
    this: *mut Vec<ddddocr::BBox>,
    index: usize,
    to: *mut ddddocr::BBox,
) {
    *to = (*this)[index]
}

#[no_mangle]
pub unsafe extern "stdcall" fn get_bbox_vec_copy(
    from: *mut Vec<ddddocr::BBox>,
    to: *mut ddddocr::BBox,
) {
    std::ptr::copy_nonoverlapping((*from).as_ptr(), to, (*from).len());
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_ddddocr(ptr: *mut ddddocr::Ddddocr<'static>) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_string(ptr: *mut std::ffi::c_char) {
    let _ = std::ffi::CString::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_character_probability(ptr: *mut ddddocr::CharacterProbability) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_bbox_vec(ptr: *mut Vec<ddddocr::BBox>) {
    let _ = Box::from_raw(ptr);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn ocr() {
        unsafe {
            let this = init_classification();

            let data = include_bytes!("../image/3.png");

            let start = std::time::Instant::now();

            let cstr = classification(this, data.as_ptr(), data.len(), false);

            let duration = start.elapsed();

            println!("耗时 {} ms", duration.as_millis());

            println!("{}", std::ffi::CString::from_raw(cstr).to_str().unwrap());
        }
    }

    #[test]
    fn det() {
        unsafe {
            let this = init_detection();

            let data = include_bytes!("../image/6.jpg");

            let start = std::time::Instant::now();

            let ptr = detection(this, data.as_ptr(), data.len());

            let duration = start.elapsed();

            println!("耗时 {} ms", duration.as_millis());

            let len = get_bbox_vec_len(ptr);

            println!("{}", len);

            for i in 0..len {
                let mut a = ddddocr::BBox {
                    x1: 0,
                    y1: 0,
                    x2: 0,
                    y2: 0,
                };

                get_bbox_vec_index(ptr, i, &mut a as *mut ddddocr::BBox);

                println!("{:?}", a);
            }
        }
    }

    #[test]
    fn cp() {
        unsafe {
            let this = init_classification();

            let data = include_bytes!("../image/3.png");

            let str = std::ffi::CString::new("1234567890").unwrap();

            set_ranges(this, str.as_ptr());

            // (*this).set_ranges(7);

            let start = std::time::Instant::now();

            let cp = classification_probability(this, data.as_ptr(), data.len(), false);

            let rt = get_ranges_text(cp, ',' as u32);

            let t = get_character_probability_text(cp);

            let len = get_character_probability_len(cp, usize::MAX);

            let mut result = Vec::with_capacity(len);

            for i in 0..len {
                let len = get_character_probability_len(cp, i);
                let mut array = Vec::<f32>::with_capacity(len);
                array.set_len(len);

                // for j in 0..len {
                //     arr2.push(get_probability(cp, i, j));
                // }

                // 比 for 快几毫秒 😂
                get_character_probability_copy(cp, i, array.as_mut_ptr());

                result.push(array)
            }

            let duration = start.elapsed();

            println!("耗时 {} ms", duration.as_millis());

            let start = std::time::Instant::now();

            let jt = character_probability_to_json(cp);

            let duration = start.elapsed();

            println!("序列化到 json 耗时 {} ms", duration.as_millis());

            println!(
                "字符集 {}",
                std::ffi::CString::from_raw(rt).to_str().unwrap()
            );

            println!(
                "识别文本 {}",
                std::ffi::CString::from_raw(t).to_str().unwrap()
            );

            println!("数组结果\n{:?}", result);

            println!(
                "字符 {:e} {:e}",
                get_character_probability(cp, 0, 0),
                get_character_probability(cp, 0, 1)
            );

            println!(
                "json 文本\n{}",
                std::ffi::CString::from_raw(jt).to_str().unwrap()
            );
        }
    }
}
