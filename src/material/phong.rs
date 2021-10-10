use super::mat_util::*;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Phong<F: Float, S: Sampler<F, F>, M: Material<F=F>>
{
    pow: S,
    mat: M,
}

impl<F: Float, S: Sampler<F, F>, M: Material<F=F>> Phong<F, S, M>
{
    pub fn new(pow: S, mat: M) -> Self
    {
        Self { pow, mat }
    }
}

impl<F: Float> Phong<F, F, Color<F>>
{
    pub fn white() -> Self
    {
        Self::new(F::from_u32(8), Color::white())
    }

    pub fn black() -> Self
    {
        Self::new(F::from_u32(8), Color::black())
    }
}

impl<F: Float, S: Sampler<F, F>, M: Material<F=F>> Material for Phong<F, S, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Box<dyn Light<F>>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut res = Color::black();

        let self_color = self.mat.render(hit, maxel, lights, rt);
        let spec_adjust = self.pow.sample(maxel.uv) / F::from_u32(2);

        for light in lights {
            let light_color = rt.ray_shadow(hit, maxel, &**light).unwrap_or_else(|| light.get_color());

            let light_vec = hit.pos.vector_to(light.get_position());
            let light_dir = light_vec.normalize();
            let refl_dir = light_dir.reflect(&maxel.normal);
            let spec_angle = -refl_dir.dot(hit.dir).clamp(F::ZERO, F::ONE);

            let light_color = light.attenuate(light_color * self_color, light_vec.magnitude());

            let lambert = maxel.normal.dot(light_dir);

            if lambert > F::BIAS {
                res += light_color * lambert;

                let specular = spec_angle.pow(F::from_u32(32));
                res += light_color * specular / spec_adjust;
            }
        }
        res
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light, rt)
    }
}
