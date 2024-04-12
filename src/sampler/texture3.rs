use image::{DynamicImage, GenericImageView, Pixel};

use crate::sampler::Sampler;
use crate::types::{Color, Float, Point};

impl<F: Float> Sampler<u32, Color<F>> for DynamicImage {
    fn sample(&self, uv: Point<u32>) -> Color<F> {
        let (w, h) = Sampler::<u32, Color<F>>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        Color::new(
            F::from_u32(c[0] as u32),
            F::from_u32(c[1] as u32),
            F::from_u32(c[2] as u32),
        ) / max
    }

    fn dimensions(&self) -> (u32, u32) {
        GenericImageView::dimensions(self)
    }

    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        let (w, h) = GenericImageView::dimensions(self);
        ui.label(name);
        ui.monospace(format!("Texture [{w}x{h}]"));
        ui.end_row();
        false
    }
}
