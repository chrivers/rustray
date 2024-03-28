use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ChessBoardXYZ<A: Material, B: Material> {
    a: A,
    b: B,
}

impl<A: Material, B: Material> ChessBoardXYZ<A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<F: Float, A: Material<F = F>, B: Material<F = F>> Material for ChessBoardXYZ<A, B> {
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let x = maxel.pos.x.abs().fract() > F::HALF;
        let y = maxel.pos.y.abs().fract() > F::HALF;
        let z = maxel.pos.z.abs().fract() > F::HALF;

        if x ^ y ^ z {
            self.a.render(maxel, rt)
        } else {
            self.b.render(maxel, rt)
        }
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Chessboard XYZ")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
