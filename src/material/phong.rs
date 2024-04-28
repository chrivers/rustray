use std::ops::Mul;

use cgmath::InnerSpace;
use num::Zero;

use crate::material::Material;
use crate::sampler::{Sampler, Texel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Point, Vectorx};

#[derive(Copy, Clone, Debug)]
pub struct Phong<F, SE, SD, SS, SP>
where
    F: Float + Texel,
    SE: Sampler<F, Color<F>>,
    SD: Sampler<F, Color<F>>,
    SS: Sampler<F, Color<F>>,
    SP: Sampler<F, F>,
{
    /// Emissive color
    ke: Color<F>,

    /// Emissive color map (texture)
    ke_map: Option<SE>,

    /// Diffuse color
    kd: Color<F>,

    /// Diffuse color map (texture)
    kd_map: Option<SD>,

    /// Specular color
    ks: Color<F>,

    /// Specular color map (texture)
    ks_map: Option<SS>,

    /// Specular power
    pow: F,

    /// Specular power map (texture)
    pow_map: Option<SP>,

    /// Ambient color
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
    pub const fn new() -> Self {
        Self {
            ambient: Color::BLACK,
            ke: Color::BLACK,
            ke_map: None,
            kd: Color::WHITE,
            kd_map: None,
            ks: Color::BLACK,
            ks_map: None,
            pow: F::ZERO,
            pow_map: None,
        }
    }

    #[must_use]
    pub fn with_ke_map(self, ke_map: Option<SE>) -> Self {
        Self { ke_map, ..self }
    }

    #[must_use]
    pub fn with_ke(self, ke: Color<F>) -> Self {
        Self { ke, ..self }
    }

    #[must_use]
    pub fn with_kd(self, kd: Color<F>) -> Self {
        Self { kd, ..self }
    }

    #[must_use]
    pub fn with_kd_map(self, kd_map: Option<SD>) -> Self {
        Self { kd_map, ..self }
    }

    #[must_use]
    pub fn with_ks(self, ks: Color<F>) -> Self {
        Self { ks, ..self }
    }

    #[must_use]
    pub fn with_ks_map(self, ks_map: Option<SS>) -> Self {
        Self { ks_map, ..self }
    }

    #[must_use]
    pub fn with_pow(self, pow: F) -> Self {
        Self { pow, ..self }
    }

    #[must_use]
    pub fn with_pow_map(self, pow_map: Option<SP>) -> Self {
        Self { pow_map, ..self }
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self { ambient, ..self }
    }

    fn sample_map<T, S>(color: T, sampler: &Option<S>, uv: Point<F>) -> T
    where
        T: Texel + Mul<T, Output = T>,
        S: Sampler<F, T>,
    {
        sampler.as_ref().map_or(color, |s| s.sample(uv) * color)
    }
}

#[allow(clippy::mismatching_type_param_order)]
impl<F: Float + Texel> Phong<F, Color<F>, Color<F>, Color<F>, F> {
    #[must_use]
    pub fn white() -> Self {
        Self {
            ke: Color::BLACK,
            ke_map: None,
            kd: Color::WHITE,
            kd_map: None,
            ks: Color::WHITE,
            ks_map: None,
            pow: F::from_u32(8),
            pow_map: None,
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

        let emis_color = Self::sample_map(self.ke, &self.ke_map, uv);
        let diff_color = Self::sample_map(self.kd, &self.kd_map, uv);
        let spec_color = Self::sample_map(self.ks, &self.ks_map, uv);
        let spec_pow = Self::sample_map(self.pow, &self.pow_map, uv);

        let ambi_color = self.ambient * rt.scene().ambient;
        let mut res = emis_color + ambi_color;

        for light in &rt.scene().lights {
            let lixel = light.contribution(maxel, rt);

            let lambert = maxel.nml().dot(lixel.dir);

            if lambert < F::BIAS {
                continue;
            }

            res += (lixel.color * diff_color) * lambert;

            if spec_color.is_zero() || spec_pow.is_zero() {
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
        use crate::gui::controls;

        ui.strong("Phong");
        ui.end_row();

        let mut res = false;
        res |= Sampler::ui(&mut self.ambient, ui, "Ambient");

        res |= controls::color(ui, &mut self.ke, "Emissive");
        res |= self
            .ke_map
            .as_mut()
            .is_some_and(|m| m.ui(ui, "Emissive map"));

        res |= controls::color(ui, &mut self.kd, "Diffuse");
        res |= self
            .kd_map
            .as_mut()
            .is_some_and(|m| m.ui(ui, "Diffuse map"));

        res |= controls::color(ui, &mut self.ks, "Specular");
        res |= self
            .ks_map
            .as_mut()
            .is_some_and(|m| m.ui(ui, "Specular map"));

        res |= self.pow.ui(ui, "Specular power");
        res |= self
            .pow_map
            .as_mut()
            .is_some_and(|m| m.ui(ui, "Specular power map"));

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
    sceneobject_impl_body!("Phong", egui_phosphor::regular::DRIBBBLE_LOGO);
}
