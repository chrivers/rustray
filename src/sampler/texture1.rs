use image::{DynamicImage, GenericImageView, Pixel};

use super::samp_util::*;

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
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        ui.label(name);
        CollapsingHeader::new("Texture")
            .default_open(true)
            .show(ui, |ui| {
                let (w, h) = GenericImageView::dimensions(self);
                ui.label(format!("{w}x{h}"));
            });
    }
}
