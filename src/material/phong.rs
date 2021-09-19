use super::mat_util::*;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Phong<F: Float, M: Material, S: Sampler<F, F>>
{
    mat: M,
    pow: S,
    _p: PhantomData<F>,
}

impl<F: Float, M: Material, S: Sampler<F, F>> Phong<F, M, S>
{
    pub fn new(pow: S, mat: M) -> Self
    {
        Self { pow, mat, _p: PhantomData {} }
    }

}

impl<F: Float> Phong<F, Color<F>, F>
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

fn attenuate<F: Float>(color: Color<F>, d: F) -> Color<F>
{
    let a = F::from_f32(0.3);
    let b = F::from_f32(0.2);
    let c = F::from_f32(0.0);
    color / (a + (b + (c * d)) * d)
}

impl<F: Float, M: Material<F=F>, S: Sampler<F, F>> Material for Phong<F, M, S>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let mut res = Color::black();

        let self_color = self.mat.render(hit, maxel, lights, rt, lvl);
        let spec_adjust = self.pow.sample(maxel.uv) / F::from_u32(2);

        for light in lights {
            if rt.ray_shadow(hit, light) {
                continue
            }

            let light_vec = hit.pos.vector_to(light.pos);
            let light_dir = light_vec.normalized();
            let refl_dir = light_dir.reflect(&maxel.normal);
            let spec_angle = -refl_dir.dot(hit.dir).clamp(F::zero(), F::one());

            let light_color = attenuate(light.color * self_color, light_vec.length());

            let lambert = maxel.normal.dot(light_dir);

            if lambert > F::BIAS {
                res += light_color * lambert;

                let specular = spec_angle.pow(F::from_u32(32));
                res += light_color * specular / spec_adjust;
            }
        }
        res
    }
}
