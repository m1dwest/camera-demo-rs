use eframe::egui;
use eframe::egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui, Vec2};

pub enum PixelFormat {
    Rgb8,
}

pub struct CameraView {
    texture: Option<TextureHandle>,
    image: ColorImage,
}

impl CameraView {
    pub fn new() -> Self {
        Self {
            texture: None,
            image: ColorImage::new([1, 1], vec![Color32::BLACK]),
        }
    }

    pub fn update_frame(
        &mut self,
        ctx: &egui::Context,
        bytes: &[u8],
        width: usize,
        height: usize,
        format: PixelFormat,
    ) {
        self.ensure_size(width, height);
        self.fill_color_image(bytes, format);

        match &mut self.texture {
            None => {
                self.texture = Some(ctx.load_texture(
                    "camera_frame",
                    self.image.clone(),
                    TextureOptions::LINEAR,
                ));
            }
            Some(tex) => {
                tex.set(self.image.clone(), TextureOptions::LINEAR);
            }
        }
    }

    pub fn show(&self, ui: &mut Ui, desired_size: Option<Vec2>) {
        let Some(tex) = &self.texture else {
            return;
        };

        let tex_id = tex.id();
        let native_size = tex.size_vec2();
        let size = desired_size.unwrap_or(native_size);

        let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        ui.painter().image(tex_id, rect, uv, Color32::WHITE);
    }

    fn ensure_size(&mut self, width: usize, height: usize) {
        let target = [width, height];
        if self.image.size != target {
            self.image = ColorImage::new(target, vec![Color32::BLACK; width * height]);
        }
    }

    fn fill_color_image(&mut self, bytes: &[u8], format: PixelFormat) {
        let w = self.image.size[0];
        let h = self.image.size[1];
        let n = w * h;

        match format {
            PixelFormat::Rgb8 => {
                assert_eq!(bytes.len(), n * 3);
                for (i, px) in bytes.chunks_exact(3).enumerate() {
                    self.image.pixels[i] = Color32::from_rgb(px[0], px[1], px[2]);
                }
            }
        }
    }
}

