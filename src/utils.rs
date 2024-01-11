type Dimensions = (u32, u32);

#[derive(Debug)]
pub struct CropArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

// 按照输入的长宽比例，计算出左右和上下居中的座标们（宽度保持最大，高度按比例决定）。
pub fn center_dimensions((width, height): Dimensions, width_scale: f64) -> CropArea {
    // 根据高度和比例计算最大宽度。
    let max_width = (width_scale * height as f64).ceil() as u32;
    // println!("width: {}, max_width: {}", width, max_width);

    let mut expect_width = max_width;
    let mut expect_height = height;

    if max_width > width {
        // 如果最大宽度图片宽度，则使用图片宽度并重新计算高度。
        expect_width = width;
        expect_height = (expect_width as f64 / width_scale).ceil() as u32;
    }

    // 计算左右居中的 x 点位置（计算留白宽度再除以 2）。
    let x = (width - expect_width) / 2;
    // 计算上下居中的 x 点位置（计算留白高度再除以 2）。
    let y = (height - expect_height) / 2;

    CropArea {
        x,
        y,
        width: expect_width,
        height: expect_height,
    }
}
