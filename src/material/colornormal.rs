use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorNormal<F: Float> {
    scale: F,
}

impl<F: Float> ColorNormal<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material for ColorNormal<F> {
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let n = maxel.nml();
        Color::new(n.x, n.y, n.z) * self.scale
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Color Normals")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
