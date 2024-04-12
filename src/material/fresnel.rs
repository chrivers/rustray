use cgmath::{InnerSpace, VectorSpace};
use num::Zero;

use crate::light::Lixel;
use crate::material::{Material, Mirror};
use crate::sampler::Sampler;
use crate::sampler::Texel;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel};

#[derive(Copy, Clone, Debug)]
pub struct Fresnel<F, SI, ST, SR>
where
    F: Float + Texel,
    SI: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    ior: SI,
    refr: ST,
    refl: Mirror<F, SR>,
}

impl<F, SI, ST, SR> Fresnel<F, SI, ST, SR>
where
    F: Float + Texel,
    SI: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    pub const fn new(ior: SI, refr: ST, refl: SR) -> Self {
        Self {
            ior,
            refl: Mirror::new(refl),
            refr,
        }
    }
}

impl<F, SI, ST, SR> Material<F> for Fresnel<F, SI, ST, SR>
where
    F: Float + Texel,
    SI: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
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

    fn shadow(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        let uv = maxel.uv();
        let sha = self.refr.sample(uv);
        let lambert = lixel.dir.dot(maxel.nml());

        sha * lixel.color * lambert
    }
}

#[cfg(feature = "gui")]
impl<F, SI, ST, SR> Interactive<F> for Fresnel<F, SI, ST, SR>
where
    F: Float + Texel,
    SI: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.strong("Fresnel");
        ui.end_row();

        let mut res = false;
        res |= self.ior.ui(ui, "Index of refraction");
        res |= self.refl.ui(ui);
        res |= self.refr.ui(ui, "Refraction");
        res
    }
}

#[cfg(feature = "gui")]
impl<F, SI, ST, SR> SceneObject<F> for Fresnel<F, SI, ST, SR>
where
    F: Float + Texel,
    SI: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    sceneobject_impl_body!("Fresnel");
}
