use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
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

    fn render(&self, maxel: &mut Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let uv = maxel.uv();
        let mut smaxel = maxel.with_uv(point!(uv.x * self.u, uv.x * self.v));
        self.mat.render(&mut smaxel, lights, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>
    {
        self.mat.shadow(maxel, light)
    }
}
