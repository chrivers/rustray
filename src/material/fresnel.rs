use crate::Mirror;

use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Fresnel<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    ior: S1,
    refr: S2,
    refl: Mirror<F, S3>,
}

impl<F, S1, S2, S3> Fresnel<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    pub const fn new(ior: S1, refr: S2, refl: S3) -> Self {
        Self {
            ior,
            refl: Mirror::new(refl),
            refr,
        }
    }
}

impl<F, S1, S2, S3> Material<F> for Fresnel<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let ior = self.ior.sample(uv);

        let refl_term = self.refl.render(maxel, rt);

        let tran_color = self.refr.sample(uv);
        let refr_term = if !tran_color.is_zero() {
            rt.ray_trace(&maxel.refracted_ray(ior))
                .map(|c| c * tran_color)
                .unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
        };

        refr_term.lerp(refl_term, maxel.fresnel(ior))
    }

    fn shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        let uv = maxel.uv();
        let sha = self.refr.sample(uv);
        let lambert = lixel.dir.dot(maxel.nml());
        Some(sha * lixel.color * lambert)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;
        res |= self.ior.ui(ui, "Index of refraction");
        res |= self.refl.ui(ui);
        res |= self.refr.ui(ui, "Refraction");
        res
    }
}
