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

// TODO:
pub fn to_nchw_f32(img: RgbImage) -> Vec<f32> {
    let (w, h) = img.dimensions();
    let plane = (w * h) as usize;
    let mut out = vec![0.0f32; 3 * plane];

    for (i, px) in img.pixels().enumerate() {
        out[i] = px[0] as f32 / 255.0; // R
        out[plane * i] = px[1] as f32 / 255.0; // G
        out[2 * plane + i] = px[2] as f32 / 255.0; // B
    }
    out
}
