use super::mat_util::*;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Phong<F: Float, M: Material, MR: AsRef<M> + Sync, S: Sampler<F, F>>
{
    mat: MR,
    pow: S,
    _m: PhantomData<M>,
    _p: PhantomData<F>,
}

impl<F: Float, M: Material, MR: AsRef<M> + Sync, S: Sampler<F, F>> Phong<F, M, MR, S>
{
    pub fn new(pow: S, mat: MR) -> Self
    {
        Self { pow, mat, _m: PhantomData {}, _p: PhantomData {} }
    }

}

impl<F: Float> Phong<F, Color<F>, Color<F>, F>
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

impl<F: Float, M: Material<F=F>, MR: AsRef<M> + Sync, S: Sampler<F, F>> Material for Phong<F, M, MR, S>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut res = Color::black();

        let self_color = self.mat.as_ref().render(hit, maxel, lights, rt);
        let spec_adjust = self.pow.sample(maxel.uv) / F::from_u32(2);

        for light in lights {
            let light_color = rt.ray_shadow(hit, maxel, light).unwrap_or(light.color);

            let light_vec = hit.pos.vector_to(light.pos);
            let light_dir = light_vec.normalize();
            let refl_dir = light_dir.reflect(&maxel.normal);
            let spec_angle = -refl_dir.dot(hit.dir).clamp(F::zero(), F::one());

            let light_color = attenuate(light_color * self_color, light_vec.magnitude());

            let lambert = maxel.normal.dot(light_dir);

            if lambert > F::BIAS {
                res += light_color * lambert;

                let specular = spec_angle.pow(F::from_u32(32));
                res += light_color * specular / spec_adjust;
            }
        }
        res
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.as_ref().shadow(hit, maxel, light, rt)
    }
}

impl<F: Float, M: Material<F=F>, MR: AsRef<M> + Sync, S: Sampler<F, F>> AsRef<Phong<F, M, MR, S>> for Phong<F, M, MR, S>
{
    fn as_ref(&self) -> &Self {
        self
    }
}
