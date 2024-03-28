use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorST<F: Float> {
    scale: F,
}

impl<F: Float> ColorST<F> {
    pub fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material for ColorST<F> {
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let st = maxel.st();
        let w = F::ONE - st.x - st.y;
        Color::new(st.x, w, st.y) * self.scale
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Color ST")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
