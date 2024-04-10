use crate::Fresnel;

use super::mat_util::*;

/// Smart material shader that supports ambient, diffuse, specular, translucent,
/// and reflective light. Implements the Phong shader model for light transport.
#[derive(Clone, Debug)]
pub struct Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    pow: S1,
    ke: S2,
    kd: S3,
    ks: S4,
    fresnel: Fresnel<F, F, S5, S6>,
    ambient: Color<F>,
}

impl<F, S1, S2, S3, S4, S5, S6> Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    #[must_use]
    pub const fn new(ior: F, pow: S1, ke: S2, kd: S3, ks: S4, kt: S5, kr: S6) -> Self {
        Self {
            pow,
            ke,
            kd,
            ks,
            fresnel: Fresnel::new(ior, kt, kr),
            ambient: Color::BLACK,
        }
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self { ambient, ..self }
    }
}

impl<F, S1, S2, S3, S4, S5, S6> Material<F> for Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let normal = maxel.nml();
        let diff_color = self.kd.sample(uv);
        let spec_color = self.ks.sample(uv);
        let spec_pow = self.pow.sample(uv);
        let ambi_color = self.ambient * rt.ambient();

        let mut res = self.ke.sample(uv) + ambi_color;

        res += self.fresnel.render(maxel, rt);

        for light in rt.get_lights() {
            let lixel = light.contribution(maxel, rt);

            let lambert = normal.dot(lixel.dir);

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

    fn shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        self.fresnel.shadow(maxel, lixel)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        CollapsingHeader::new("Smart")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let mut res = false;
                        res |= Sampler::ui(&mut self.ambient, ui, "Ambient");
                        res |= self.ke.ui(ui, "Emissive");
                        res |= self.kd.ui(ui, "Diffuse");
                        res |= self.pow.ui(ui, "Specular power");
                        res |= self.ks.ui(ui, "Specular");
                        res |= self.fresnel.ui(ui);
                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}
