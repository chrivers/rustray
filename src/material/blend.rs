use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Blend<F: Float, A: Material, B: Material> {
    a: A,
    b: B,
    pct: F,
}

impl<F: Float, A: Material, B: Material> Blend<F, A, B> {
    pub fn new(a: A, b: B, pct: F) -> Self {
        Self { a, b, pct }
    }
}

impl<F: Float, A: Material<F = F>, B: Material<F = F>> Material for Blend<F, A, B> {
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let a = self.a.render(maxel, rt);
        let b = self.b.render(maxel, rt);
        a.lerp(b, self.pct)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Blend")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
