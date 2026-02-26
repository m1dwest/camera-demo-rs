use anyhow::{Context, Result};
use image::{Rgb, RgbImage};

const LETTERBOX_COLOR: Rgb<u8> = Rgb([114, 114, 114]);

pub fn letterbox(data: Vec<u8>, w: u32, h: u32, target: u32) -> Result<RgbImage> {
    let src =
        RgbImage::from_raw(w, h, data).context("Unable to create an RgbImage from raw data")?;

    let ratio_w = target as f32 / w as f32;
    let ratio_h = target as f32 / h as f32;
    let ratio = ratio_w.min(ratio_h);

    let new_w = ((w as f32 * ratio).round() as u32).max(1);
    let new_h = ((h as f32 * ratio).round() as u32).max(1);

    let resized =
        image::imageops::resize(&src, new_w, new_h, image::imageops::FilterType::Triangle);
    let mut out = RgbImage::from_pixel(target, target, LETTERBOX_COLOR);
    let pad_x = (target - new_w) / 2;
    let pad_y = (target - new_h) / 2;

    image::imageops::overlay(&mut out, &resized, pad_x.into(), pad_y.into());

    Ok(out)
}

pub fn input_array(rgb: RgbImage) -> ndarray::Array4<f32> {
    let mut input = ndarray::Array::zeros((1, 3, rgb.width() as usize, rgb.height() as usize));
    for (i, px) in rgb.pixels().enumerate() {
        let y = i as u32 / rgb.height();
        let x = i as u32 - (y * rgb.width());
        let r = px[0] as f32 / 255.0;
        let g = px[1] as f32 / 255.0;
        let b = px[2] as f32 / 255.0;
        input[[0, 0, y as usize, x as usize]] = r;
        input[[0, 1, y as usize, x as usize]] = g;
        input[[0, 2, y as usize, x as usize]] = b;
    }

    input
}
