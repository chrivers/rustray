use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorUV<F: Float> {
    scale: F,
}

impl<F: Float> ColorUV<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material<F> for ColorUV<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        Color::new(uv.x, F::ZERO, uv.y) * self.scale
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        CollapsingHeader::new("Color UV")
            .default_open(true)
            .show(ui, |_ui| {});
        false
    }
}
