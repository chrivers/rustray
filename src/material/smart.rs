use crate::light::Lixel;
use crate::material::{Fresnel, Material, Phong};
use crate::sampler::{Sampler, Texel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel};

/// Smart material shader that supports ambient, diffuse, specular, translucent,
/// and reflective light. Implements the Phong shader model for light transport.
#[derive(Clone, Debug)]
pub struct Smart<F, SE, SD, SS, SP, ST, SR>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    phong: Phong<F, SE, SD, SS, SP>,
    fresnel: Fresnel<F, F, ST, SR>,
}

impl<F, SE, SD, SS, SP, ST, SR> Smart<F, SE, SD, SS, SP, ST, SR>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    #[must_use]
    pub const fn new(ior: F, pow: SP, ke: SE, kd: SD, ks: SS, kt: ST, kr: SR) -> Self {
        Self {
            phong: Phong::new(ke, kd, ks, pow),
            fresnel: Fresnel::new(ior, kt, kr),
        }
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self {
            phong: self.phong.with_ambient(ambient),
            ..self
        }
    }
}

impl<F, SE, SD, SS, SP, ST, SR> Material<F> for Smart<F, SE, SD, SS, SP, ST, SR>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        self.phong.render(maxel, rt) + self.fresnel.render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        self.fresnel.shadow(maxel, rt, lixel)
    }
}

#[cfg(feature = "gui")]
impl<F, SE, SD, SS, SP, ST, SR> Interactive<F> for Smart<F, SE, SD, SS, SP, ST, SR>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;
        res |= self.phong.ui(ui);
        res |= self.fresnel.ui(ui);
        res
    }
}

impl<F, SE, SD, SS, SP, ST, SR> SceneObject<F> for Smart<F, SE, SD, SS, SP, ST, SR>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
    ST: Sampler<F, Color<F>>,
    SR: Sampler<F, Color<F>>,
{
    sceneobject_impl_body!("Smart material");
}
