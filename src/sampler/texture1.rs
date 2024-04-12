use image::{DynamicImage, GenericImageView, Pixel};

use crate::sampler::{Sampler, Texel};
use crate::types::{Float, Point};

impl<F: Float + Texel> Sampler<u32, F> for DynamicImage {
    fn sample(&self, uv: Point<u32>) -> F {
        let (w, h) = Sampler::<u32, F>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        F::from_u32(c[0] as u32) / max
    }

    fn dimensions(&self) -> (u32, u32) {
        GenericImageView::dimensions(self)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        let (w, h) = GenericImageView::dimensions(self);
        ui.label(name);
        ui.monospace(format!("Texture [{w}x{h}]"));
        ui.end_row();
        false
    }
}
