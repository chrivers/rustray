use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct ScaleUV<F: Float, M: Material<F=F>>
{
    u: F,
    v: F,
    mat: M,
}

impl<F: Float, M: Material<F=F>> ScaleUV<F, M>
{
    pub fn new(u: F, v: F, mat: M) -> Self
    {
        Self { u, v, mat }
    }

}

impl<F: Float, M: Material<F=F>> Material for ScaleUV<F, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut smaxel = *maxel;
        smaxel.uv.x *= self.u;
        smaxel.uv.y *= self.v;
        self.mat.render(hit, &smaxel, lights, rt)
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light, rt)
    }
}
