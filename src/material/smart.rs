use super::mat_util::*;

use num_traits::Zero;

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
    ior: F,
    pow: S1,
    ke: S2,
    kd: S3,
    ks: S4,
    kt: S5,
    kr: S6,
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
            ior,
            pow,
            ke,
            kd,
            ks,
            kt,
            kr,
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

        let tran_color = self.kt.sample(uv);
        let refl_color = self.kr.sample(uv);

        let refl_term = if !refl_color.is_zero() {
            rt.ray_trace(&maxel.reflected_ray())
                .map(|c| c * refl_color)
                .unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
        };

        let ior = self.ior.sample(uv);

        let refr_term = if !tran_color.is_zero() {
            rt.ray_trace(&maxel.refracted_ray(ior))
                .map(|c| c * tran_color)
                .unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
        };

        res += refr_term.lerp(refl_term, maxel.fresnel(ior));

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
        let uv = maxel.uv();
        let sha = self.kt.sample(uv);
        let lambert = lixel.dir.dot(maxel.nml());
        Some(sha * lixel.color * lambert)
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
                        res |= self.pow.ui(ui, "Power");
                        res |= Sampler::ui(&mut self.ior, ui, "Index of refraction");
                        res |= self.ke.ui(ui, "Emissive");
                        res |= self.kd.ui(ui, "Diffuse");
                        res |= self.ks.ui(ui, "Specular");
                        res |= self.kt.ui(ui, "Translucense");
                        res |= self.kr.ui(ui, "Reflection");
                        res |= Sampler::ui(&mut self.ambient, ui, "ambient");
                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}
