use super::mat_util::*;

use num_traits::Zero;

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
    pub fn new(ior: F, pow: S1, ke: S2, kd: S3, ks: S4, kt: S5, kr: S6) -> Self
    {
        Self { ior, pow, ke, kd, ks, kt, kr, ambient: Color::black() }
    }

    pub fn with_ambient(self, ambient: Color<F>) -> Self
    {
        Self { ambient, ..self }
    }

}

impl<F, S1, S2, S3, S4, S5, S6> Material for Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let uv = maxel.uv();
        let normal = maxel.nml();
        let diff_color = self.kd.sample(uv);
        let spec_color = self.ks.sample(uv);
        let spec_pow   = self.pow.sample(uv);
        let ambi_color = self.ambient * rt.ambient();

        let mut res = self.ke.sample(uv) + ambi_color;

        let tran_color = self.kt.sample(uv);
        let refl_color = self.kr.sample(uv);

        let refl_term = if !refl_color.is_zero() {
            let refl = maxel.reflected_ray();
            rt.ray_trace(&refl).unwrap_or_else(|| rt.background()) * refl_color
        } else {
            Color::black()
        };

        let ior = self.ior.sample(uv);

        let refr_term = if !tran_color.is_zero() {
            let refr = maxel.refracted_ray(ior);
            rt.ray_trace(&refr).unwrap_or_else(|| rt.background()) * tran_color
        } else {
            Color::black()
        };

        let fresnel = maxel.fresnel(ior);

        res += refr_term.lerp(refl_term, fresnel);

        for light in lights {
            let light_color = rt.ray_shadow(maxel, *light).unwrap_or_else(|| light.get_color());

            let light_vec = maxel.pos.vector_to(light.get_position());
            let light_dir = light_vec.normalize();
            let refl_dir = light_dir.reflect(&normal);
            let spec_angle = refl_dir.dot(maxel.dir).clamp(F::ZERO, F::ONE);

            let light_color = light.attenuate(light_color, light_vec.magnitude());

            let lambert = normal.dot(light_dir);

            if lambert > F::BIAS {
                res += (light_color * diff_color) * lambert;

                let specular = spec_angle.pow(spec_pow);
                res += (light_color * spec_color) * specular;
            }
        }
        res
    }

    fn shadow(&self, maxel: &mut Maxel<F>, _light: &dyn Light<F>) -> Option<Color<F>>
    {
        let uv = maxel.uv();
        let sha = self.kt.sample(uv);

        if sha.is_zero() {
            None
        } else {
            Some(sha)
        }
    }
}
