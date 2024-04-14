use cgmath::InnerSpace;
use num::Zero;

use crate::material::Material;
use crate::sampler::{Sampler, Texel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Vectorx};

#[derive(Copy, Clone, Debug)]
pub struct Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    ke: SE,
    kd: SD,
    ks: SS,
    pow: SP,
    ambient: Color<F>,
}

impl<F, SE, SD, SS, SP> Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    #[must_use]
    pub const fn new(ke: SE, kd: SD, ks: SS, pow: SP) -> Self {
        Self {
            ke,
            kd,
            ks,
            pow,
            ambient: Color::BLACK,
        }
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self { ambient, ..self }
    }
}

#[allow(clippy::mismatching_type_param_order)]
impl<F: Float + Texel> Phong<F, Color<F>, Color<F>, Color<F>, F> {
    #[must_use]
    pub fn white() -> Self {
        Self {
            ke: Color::BLACK,
            kd: Color::WHITE,
            ks: Color::WHITE,
            pow: F::from_u32(8),
            ambient: Color::BLACK,
        }
    }

    #[must_use]
    pub fn black() -> Self {
        Self {
            ke: Color::BLACK,
            kd: Color::BLACK,
            ks: Color::WHITE,
            pow: F::from_u32(8),
            ambient: Color::BLACK,
        }
    }
}

impl<F, SE, SD, SS, SP> Material<F> for Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();

        let diff_color = self.kd.sample(uv);
        let spec_color = self.ks.sample(uv);
        let spec_pow = self.pow.sample(uv);

        let ambi_color = self.ambient * rt.scene().ambient;
        let mut res = self.ke.sample(uv) + ambi_color;

        for light in &rt.scene().lights {
            let lixel = light.contribution(maxel, rt);

            let lambert = maxel.nml().dot(lixel.dir);

            if lambert < F::BIAS {
                continue;
            }

            res += (lixel.color * diff_color) * lambert;

            if spec_color.is_zero() {
                continue;
            }

            let refl_dir = lixel.dir.reflect(&maxel.nml());
            let spec_angle = refl_dir.dot(maxel.dir).max(F::ZERO);
            let specular = spec_angle.pow(spec_pow);

            res += lixel.color * spec_color * specular;
        }
        res
    }
}

impl<F, SE, SD, SS, SP> Interactive<F> for Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.strong("Phong");
        ui.end_row();

        let mut res = false;
        res |= Sampler::ui(&mut self.ambient, ui, "Ambient");
        res |= self.ke.ui(ui, "Emissive");
        res |= self.kd.ui(ui, "Diffuse");
        res |= self.pow.ui(ui, "Specular power");
        res |= self.ks.ui(ui, "Specular");
        res
    }
}

impl<F, SE, SD, SS, SP> SceneObject<F> for Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    sceneobject_impl_body!("Phong");
}
