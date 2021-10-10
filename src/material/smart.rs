use super::mat_util::*;
use std::marker::PhantomData;

use std::sync::Arc;

use num_traits::Zero;

#[derive(Clone)]
pub struct Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float,
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
}

impl<F, S1, S2, S3, S4, S5, S6> Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    pub fn new(ior: F, pow: S1, ke: S2, kd: S3, ks: S4, kt: S5, kr: S6) -> Self
    {
        Self { ior, pow, ke, kd, ks, kt, kr }
    }

}

impl<F, S1, S2, S3, S4, S5, S6> Material for Smart<F, S1, S2, S3, S4, S5, S6>
where
    F: Float,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, Color<F>>,
    S5: Sampler<F, Color<F>>,
    S6: Sampler<F, Color<F>>,
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Box<dyn Light<F>>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let diff_color = self.kd.sample(maxel.uv);
        let spec_color = self.ks.sample(maxel.uv);
        let spec_pow   = self.pow.sample(maxel.uv);

        let mut res = self.ke.sample(maxel.uv);

        let tran_color = self.kt.sample(maxel.uv);
        let refl_color = self.kr.sample(maxel.uv);

        if !refl_color.is_zero() {
            let refl = hit.reflected_ray(&maxel.normal);
            res += rt.ray_trace(&refl).unwrap_or_else(Color::black) * refl_color
        }

        if !tran_color.is_zero() {
            let ior = self.ior.sample(maxel.uv);
            let refr = hit.refracted_ray(&maxel.normal, ior);
            res += rt.ray_trace(&refr).unwrap_or_else(Color::black) * tran_color
        }

        for light in lights {
            let light_color = rt.ray_shadow(hit, maxel, &**light).unwrap_or_else(|| light.get_color());

            let light_vec = hit.pos.vector_to(light.get_position());
            let light_dir = light_vec.normalize();
            let refl_dir = light_dir.reflect(&maxel.normal);
            let spec_angle = refl_dir.dot(hit.dir).clamp(F::ZERO, F::ONE);

            let light_color = light.attenuate(light_color, light_vec.magnitude());

            let lambert = maxel.normal.dot(light_dir);

            if lambert > F::BIAS {
                res += (light_color * diff_color) * lambert;

                let specular = spec_angle.pow(spec_pow);
                res += (light_color * spec_color) * specular;
            }
        }
        res
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        let sha = self.kt.sample(maxel.uv);

        if sha.is_zero() {
            None
        } else {
            Some(sha)
        }
    }
}
