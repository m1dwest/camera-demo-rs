use eframe::egui;
use eframe::egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui, Vec2};

use crate::core::PixelFormat;
use crate::core::{Frame, Intrinsics};

fn scale_to_fit(to_scale: &Vec2, fit_into: &Vec2) -> Vec2 {
    let ratio = (fit_into.x / to_scale.x).min(fit_into.y / to_scale.y);
    Vec2::new(to_scale.x * ratio, to_scale.y * ratio)
}

pub struct CameraView {
    base_tex: Option<TextureHandle>,
    overlay_tex: Option<TextureHandle>,
    image: ColorImage,
}

impl CameraView {
    pub fn new() -> Self {
        Self {
            base_tex: None,
            overlay_tex: None,
            image: ColorImage::new([1, 1], vec![Color32::BLACK]),
        }
    }

    pub fn update_frame(
        &mut self,
        ctx: &egui::Context,
        frame: Frame,
        overlay: Option<Frame>,
        intrinsics: Intrinsics,
    ) {
        let size = [intrinsics.width, intrinsics.height];
        let color_image = egui::ColorImage::from_rgb(size, frame.as_slice());
        let base_tex = self.base_tex.get_or_insert_with(|| {
            ctx.load_texture("video_frame", color_image.clone(), Default::default())
        });

        base_tex.set(color_image, egui::TextureOptions::LINEAR);

        if let Some(overlay) = overlay {
            let overlay_image = egui::ColorImage::from_rgba_unmultiplied(size, overlay.as_slice());
            let overlay_tex = self.overlay_tex.get_or_insert_with(|| {
                ctx.load_texture(
                    "video_frame_overlay",
                    overlay_image.clone(),
                    TextureOptions::LINEAR,
                )
            });

            overlay_tex.set(overlay_image, TextureOptions::LINEAR);
        } else {
            self.overlay_tex = None;
        }

        // self.ensure_size(width, height);
        // self.fill_color_image(bytes, format);
        //
        // match &mut self.base_tex {
        //     None => {
        //         self.base_tex = Some(ctx.load_texture(
        //             "camera_frame",
        //             self.image.clone(),
        //             TextureOptions::LINEAR,
        //         ));
        //     }
        //     Some(tex) => {
        //         tex.set(self.image.clone(), TextureOptions::LINEAR);
        //     }
        // }
    }

    pub fn show(&mut self, ui: &mut Ui, desired_size: Option<Vec2>) {
        // if let Some(tex) = self.base_tex {
        //     ui.image(tex);
        // }
        let Some(tex) = &self.base_tex else {
            return;
        };

        let tex_id = tex.id();
        let native_size = tex.size_vec2();
        let size = desired_size
            .map(|s| scale_to_fit(&native_size, &s))
            .unwrap_or(native_size);

        let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        ui.painter().image(tex_id, rect, uv, Color32::WHITE);

        if let Some(overlay) = &self.overlay_tex {
            ui.painter().image(overlay.id(), rect, uv, Color32::WHITE);
        }
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
