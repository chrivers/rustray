use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Fresnel<F: Float + Texel, S: Sampler<F, F>> {
    ior: S,
    _p: PhantomData<F>,
}

impl<F: Float + Texel, S: Sampler<F, F>> Fresnel<F, S> {
    pub const fn new(ior: S) -> Self {
        Self {
            ior,
            _p: PhantomData {},
        }
    }
}

impl<F: Float + Texel, S: Sampler<F, F>> Fresnel<F, S> {
    pub fn trace_fresnel(
        maxel: &mut Maxel<F>,
        ior: F,
        rt: &dyn RayTracer<F>,
    ) -> (Option<Color<F>>, Option<Color<F>>) {
        let refl = rt.ray_trace(&maxel.reflected_ray());
        let refr = rt.ray_trace(&maxel.refracted_ray(ior));

        (refl, refr)
    }

    pub fn blend_fresnel(
        maxel: &mut Maxel<F>,
        ior: F,
        c_refl: Color<F>,
        c_refr: Color<F>,
    ) -> Color<F> {
        let fr = maxel.fresnel(ior);

        c_refr.lerp(c_refl, fr)
    }
}

impl<F: Float + Texel, S: Sampler<F, F>> Material<F> for Fresnel<F, S> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let ior = self.ior.sample(maxel.uv());

        let (refl, refr) = Self::trace_fresnel(maxel, ior, rt);
        let c_refl = refl.unwrap_or(Color::BLACK);
        let c_refr = refr.unwrap_or(Color::BLACK);

        Self::blend_fresnel(maxel, ior, c_refl, c_refr)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        CollapsingHeader::new("Fresnel")
            .default_open(true)
            .show(ui, |ui| -> bool {
                self.ior.ui(ui, "Index of refraction");
                false
            })
            .body_returned
            .unwrap_or(false)
    }
}
