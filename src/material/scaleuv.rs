use super::mat_util::*;

/// Proxy material that scales UV coordinates, before rendering backing material.
#[derive(Copy, Clone, Debug)]
pub struct ScaleUV<F: Float, M: Material<F>> {
    uv: Point<F>,
    mat: M,
}

impl<F: Float, M: Material<F>> ScaleUV<F, M> {
    pub fn new(u: F, v: F, mat: M) -> Self {
        Self {
            uv: point!(u, v),
            mat,
        }
    }
}

impl<F: Float, M: Material<F>> Material<F> for ScaleUV<F, M> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let mut smaxel = maxel.with_uv(self.uv.dot(uv));
        self.mat.render(&mut smaxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        self.mat.shadow(maxel, lixel)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("ScaleUV")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    Slider::new(&mut self.uv.x, F::ZERO..=F::from_u32(50))
                        .logarithmic(true)
                        .clamp_to_range(false)
                        .trailing_fill(true)
                        .text("u scaling factor"),
                );
                ui.add(
                    Slider::new(&mut self.uv.y, F::ZERO..=F::from_u32(50))
                        .logarithmic(true)
                        .clamp_to_range(false)
                        .trailing_fill(true)
                        .text("v scaling factor"),
                );
                self.mat.ui(ui);
            });
    }
}
