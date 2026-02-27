use anyhow::{Context, Result};
use image::{Rgb, RgbImage};

use crate::app::vision;
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};

pub struct Letterbox {
    pub rgb: RgbImage,

    ratio: f32,
    pad_x: u32,
    pad_y: u32,
}

impl Letterbox {
    const BG_COLOR: Rgb<u8> = Rgb([114, 114, 114]);

    pub fn from_vec(data: Vec<u8>, w: u32, h: u32, target: u32) -> Result<Self> {
        let src =
            RgbImage::from_raw(w, h, data).context("Unable to create an RgbImage from raw data")?;

        let ratio_w = target as f32 / w as f32;
        let ratio_h = target as f32 / h as f32;
        let ratio = ratio_w.min(ratio_h);

        let new_w = ((w as f32 * ratio).round() as u32).max(1);
        let new_h = ((h as f32 * ratio).round() as u32).max(1);

        let resized =
            image::imageops::resize(&src, new_w, new_h, image::imageops::FilterType::Triangle);
        let mut out = RgbImage::from_pixel(target, target, Letterbox::BG_COLOR);
        let pad_x = (target - new_w) / 2;
        let pad_y = (target - new_h) / 2;

        image::imageops::overlay(&mut out, &resized, pad_x.into(), pad_y.into());

        let letterbox = Self {
            rgb: out,
            ratio,
            pad_x,
            pad_y,
        };

        Ok(letterbox)
    }

    pub fn yolo_rect_to_src(&self, rect: &vision::Rect) -> vision::Rect {
        let cx = rect.x - self.pad_x as f32;
        let cy = rect.y - self.pad_y as f32;

        let cx = (cx / self.ratio).round();
        let cy = (cy / self.ratio).round();
        let w = rect.w / self.ratio;
        let h = rect.h / self.ratio;

        let x = cx - w / 2.0;
        let y = cy - h / 2.0;

        vision::Rect { x, y, w, h }
    }
}

pub fn input_array(rgb: &RgbImage) -> ndarray::Array4<f32> {
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

// TODO: move to vision
pub fn generate_overlay(
    width: u32,
    height: u32,
    detections: Vec<vision::Detection>,
) -> anyhow::Result<crate::core::Frame> {
    use std::include_bytes;

    let mut buffer = vec![0u8; width as usize * height as usize * 4];
    let mut overlay = ImageBuffer::from_raw(width, height, buffer.as_mut_slice())
        .expect("buffer length matches dimensions");

    let font_data: &[u8] = include_bytes!("../../assets/static/RobotoMono-Bold.ttf");
    let font = ab_glyph::FontRef::try_from_slice(font_data).context("Unable to load font")?;

    let text_scale = ab_glyph::PxScale { x: 18.0, y: 18.0 };
    let text_color = image::Rgba([0, 255, 0, 255]);

    for d in &detections {
        let rect =
            Rect::at(d.rect.x as i32, d.rect.y as i32).of_size(d.rect.w as u32, d.rect.h as u32);
        let text = d.label.clone().unwrap_or("no label".to_owned());
        draw_text_mut(
            &mut overlay,
            text_color,
            (d.rect.x + 10.0) as i32,
            (d.rect.y + 10.0) as i32,
            text_scale,
            &font,
            &text,
        );
        draw_rect_thick(&mut overlay, rect, text_color, 5);
    }
    let data: Vec<u8> = overlay.as_raw().to_vec();
    Ok(crate::core::Frame::from_vec(data))
}

use image::{ImageBuffer, Rgba};
use imageproc::rect::Rect;

pub fn draw_rect_thick(
    img: &mut ImageBuffer<Rgba<u8>, &mut [u8]>,
    rect: Rect,
    color: Rgba<u8>,
    thickness: u32,
) {
    for t in 0..thickness {
        let x = rect.left().saturating_sub(t as i32);
        let y = rect.top().saturating_sub(t as i32);
        let w = rect.width() + 2 * t;
        let h = rect.height() + 2 * t;

        let r = Rect::at(x, y).of_size(w, h);
        draw_hollow_rect_mut(img, r, color);
    }
}
