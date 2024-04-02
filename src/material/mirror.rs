use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Mirror<F, S>
where
    F: Float + Texel,
    S: Sampler<F, F>,
{
    refl: S,
    _p: PhantomData<F>,
}

impl<F: Float + Texel, S: Sampler<F, F>> Mirror<F, S> {
    pub const fn new(refl: S) -> Self {
        Self {
            refl,
            _p: PhantomData {},
        }
    }
}

impl<F: Float + Texel, S: Sampler<F, F>> Material<F> for Mirror<F, S> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let c_refl = maxel
            .reflected_ray()
            .and_then(|refl| rt.ray_trace(&refl))
            .unwrap_or_else(|| rt.background());

        c_refl * self.refl.sample(maxel.uv())
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Mirror")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
