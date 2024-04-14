use crate::sampler::Sampler;
use crate::types::{Color, Float, Point};
use image::{DynamicImage, GenericImageView, Pixel};

impl<F: Float> Sampler<u32, F> for DynamicImage {
    fn sample(&self, uv: Point<u32>) -> F {
        let (w, h) = Sampler::<u32, F>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        F::from_u32(u32::from(c[0])) / max
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

impl<F: Float> Sampler<u32, Color<F>> for DynamicImage {
    fn sample(&self, uv: Point<u32>) -> Color<F> {
        let (w, h) = Sampler::<u32, Color<F>>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        Color::new(
            F::from_u32(u32::from(c[0])),
            F::from_u32(u32::from(c[1])),
            F::from_u32(u32::from(c[2])),
        ) / max
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
