use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Bumpmap<F: Float, S: Sampler<F, Vector<F>>, M: Material<F=F>>
{
    pow: F,
    img: S,
    mat: M,
}

impl<F: Float, S: Sampler<F, Vector<F>>, M: Material<F=F>> Bumpmap<F, S, M>
{
    pub fn new(pow: F, img: S, mat: M) -> Self
    {
        Self { pow, img, mat }
    }
}

impl<'a, F: Float, S: Sampler<F, Vector<F>>, M: Material<F=F>> Material for Bumpmap<F, S, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let n = self.img.sample(maxel.uv);

        let mut mxl = *maxel;

        let nx =
            mxl.normalu * n.x +
            mxl.normalv * n.y +
            mxl.normal  * n.z / (self.pow + F::BIAS);

        mxl.normal = nx.normalize();

        self.mat.render(hit, &mxl, lights, rt)
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light, rt)
    }
}
