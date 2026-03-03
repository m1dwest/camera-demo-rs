use ab_glyph::ScaleFont;
use anyhow::{Context, Result};
use image::{ImageBuffer, Rgb, RgbImage, Rgba};

use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn area(&self) -> f32 {
        self.w * self.h
    }

    pub fn x2(&self) -> f32 {
        self.x + self.w
    }

    pub fn y2(&self) -> f32 {
        self.y + self.h
    }

    pub fn iou(&self, other: &Rect) -> f32 {
        let ix1 = self.x.max(other.x);
        let iy1 = self.y.max(other.y);
        let ix2 = self.x2().min(other.x2());
        let iy2 = self.y2().min(other.y2());

        let iw = (ix2 - ix1).max(0.0);
        let ih = (iy2 - iy1).max(0.0);
        let inter = iw * ih;

        let union = self.area() * other.area() - inter;
        if union <= 0.0 { 0.0 } else { inter / union }
    }
}

pub struct Letterbox {
    pub rgb: RgbImage,

    ratio: f32,
    pad_x: f32,
    pad_y: f32,
}

impl Letterbox {
    const BG_COLOR: Rgb<u8> = Rgb([114, 114, 114]);

    pub fn from_vec(data: Vec<u8>, w: u32, h: u32, target: u32) -> Result<Self> {
        let src =
            RgbImage::from_raw(w, h, data).context("Unable to create an RgbImage from raw data")?;

        let ratio_w = target as f32 / w as f32;
        let ratio_h = target as f32 / h as f32;
        let ratio = ratio_w.min(ratio_h);

        let new_w = (w as f32 * ratio).round().max(1.0);
        let new_h = (h as f32 * ratio).round().max(1.0);

        let resized = image::imageops::resize(
            &src,
            new_w as u32,
            new_h as u32,
            image::imageops::FilterType::Triangle,
        );
        let mut out = RgbImage::from_pixel(target, target, Letterbox::BG_COLOR);
        let pad_x = (target as f32 - new_w) / 2.0;
        let pad_y = (target as f32 - new_h) / 2.0;

        image::imageops::overlay(&mut out, &resized, pad_x as i64, pad_y as i64);

        let letterbox = Self {
            rgb: out,
            ratio,
            pad_x,
            pad_y,
        };

        Ok(letterbox)
    }

    pub fn yolo_rect_to_src(&self, rect: &Rect) -> Rect {
        let cx = rect.x - self.pad_x;
        let cy = rect.y - self.pad_y;

        let cx = (cx / self.ratio).round();
        let cy = (cy / self.ratio).round();
        let w = rect.w / self.ratio;
        let h = rect.h / self.ratio;

        let x = cx - w / 2.0;
        let y = cy - h / 2.0;

        Rect { x, y, w, h }
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

pub fn generate_overlay(
    width: u32,
    height: u32,
    detections: Vec<crate::core::vision::Detection>,
) -> anyhow::Result<crate::core::Frame> {
    let buffer = vec![0u8; width as usize * height as usize * 4];
    let mut overlay =
        ImageBuffer::from_vec(width, height, buffer).expect("buffer length matches dimensions");

    let color = image::Rgba([255, 0, 0, 255]);
    let box_thickness = 4;
    let text_color = image::Rgba([255, 255, 255, 255]);
    let text_painter = TextPainter::new(text_color, 18.0, 18.0)?;

    // TODO: enable all detections
    // for d in &detections {
    if let Some(d) = detections.first() {
        let box_x = d.rect.x as i32;
        let box_y = d.rect.y as i32;
        let box_w = d.rect.w as u32;
        let box_h = d.rect.h as u32;

        let rect = imageproc::rect::Rect::at(box_x, box_y).of_size(box_w, box_h);

        let label = d.label.as_deref().unwrap_or("Unknown");
        let text = format!("{}: {:.2}", label, d.score);
        let mut text_h_offset: i32 = 0;
        if let Some(text_bounds) = text_painter.measure_text_ink_bounds(&text) {
            let text_w = text_bounds.width() as u32;
            let text_h = text_bounds.height() as u32;
            let text_rect = imageproc::rect::Rect::at(
                box_x - box_thickness + 1,
                box_y - text_h as i32 - box_thickness + 1,
            )
            .of_size(
                text_w + box_thickness as u32 * 2,
                text_h + box_thickness as u32,
            );
            draw_filled_rect_mut(&mut overlay, text_rect, color);

            text_h_offset = text_h as i32 + box_thickness + 1;
        }
        overlay = draw_rect_thick(overlay, rect, color, box_thickness as u32);
        overlay = text_painter.draw_text(overlay, box_x, box_y - text_h_offset, &text);
    }
    let data: Vec<u8> = overlay.as_raw().to_vec();
    Ok(crate::core::Frame::from_vec(data))
}

pub fn draw_rect_thick(
    mut canvas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    rect: imageproc::rect::Rect,
    color: Rgba<u8>,
    thickness: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    for t in 0..thickness {
        let x = rect.left().saturating_sub(t as i32);
        let y = rect.top().saturating_sub(t as i32);
        let w = rect.width() + 2 * t;
        let h = rect.height() + 2 * t;

        let r = imageproc::rect::Rect::at(x, y).of_size(w, h);
        draw_hollow_rect_mut(&mut canvas, r, color);
    }
    canvas
}

struct TextPainter {
    color: image::Rgba<u8>,
    scale: ab_glyph::PxScale,
    font: ab_glyph::FontRef<'static>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}
impl TextBounds {
    pub fn width(&self) -> f32 {
        (self.max_x - self.min_x).max(0.0)
    }
    pub fn height(&self) -> f32 {
        (self.max_y - self.min_y).max(0.0)
    }
}

impl TextPainter {
    fn new(color: image::Rgba<u8>, scale_x: f32, scale_y: f32) -> anyhow::Result<Self> {
        use std::include_bytes;

        let font_data: &[u8] = include_bytes!("../../../assets/static/RobotoMono-Bold.ttf");
        let font = ab_glyph::FontRef::try_from_slice(font_data).context("Unable to load font")?;

        let scale = ab_glyph::PxScale {
            x: scale_x,
            y: scale_y,
        };

        Ok(Self { color, scale, font })
    }

    fn draw_text(
        &self,
        mut canvas: ImageBuffer<Rgba<u8>, Vec<u8>>,
        x: i32,
        y: i32,
        text: &str,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        draw_text_mut(&mut canvas, self.color, x, y, self.scale, &self.font, &text);
        canvas
    }

    fn measure_text_ink_bounds(&self, text: &str) -> Option<TextBounds> {
        use ab_glyph::{Font, Point};

        let scaled = self.font.as_scaled(self.scale);

        let mut caret_x: f32 = 0.0;

        let mut bounds: Option<TextBounds> = None;

        for c in text.chars() {
            let glyph_id = scaled.glyph_id(c);
            if glyph_id.0 == 0 {
                caret_x += scaled.h_advance(glyph_id);
                continue;
            }

            let mut g = glyph_id.with_scale(self.scale);
            g.position = Point { x: caret_x, y: 0.0 };

            if let Some(r) = scaled.outline_glyph(g).map(|og| og.px_bounds()) {
                let b = TextBounds {
                    min_x: r.min.x,
                    min_y: r.min.y,
                    max_x: r.max.x,
                    max_y: r.max.y,
                };

                bounds = Some(match bounds {
                    None => b,
                    Some(acc) => TextBounds {
                        min_x: acc.min_x.min(b.min_x),
                        min_y: acc.min_y.min(b.min_y),
                        max_x: acc.max_x.max(b.max_x),
                        max_y: acc.max_y.max(b.max_y),
                    },
                });
            }

            caret_x += scaled.h_advance(glyph_id);
        }

        bounds
    }
}
