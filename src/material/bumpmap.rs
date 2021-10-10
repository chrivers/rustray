use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Bumpmap<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>, MR: AsRef<M>>
{
    pow: F,
    img: S,
    mat: MR,
    _m: PhantomData<M>,
}

impl<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>, MR: AsRef<M>> Bumpmap<F, S, M, MR>
{
    pub fn new(pow: F, img: S, mat: MR) -> Self
    {
        Self { pow, img, mat, _m: PhantomData {} }
    }
}

impl<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>, MR: AsRef<M> + Sync> Material for Bumpmap<F, S, M, MR>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Box<dyn Light<F>>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut n = self.img.sample(maxel.uv);
        n.r -= F::HALF;
        n.g -= F::HALF;
        n.r *= F::TWO;
        n.g *= F::TWO;

        let mut mxl = *maxel;

        let nx =
            mxl.normalu * n.r +
            mxl.normalv * n.g +
            mxl.normal  * n.b / (self.pow + F::BIAS);

        mxl.normal = nx.normalize();

        self.mat.as_ref().render(hit, &mxl, lights, rt)
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.as_ref().shadow(hit, maxel, light, rt)
    }
}

impl<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>, MR: AsRef<M> + Sync> AsRef<Bumpmap<F, S, M, MR>> for Bumpmap<F, S, M, MR>
{
    fn as_ref(&self) -> &Self {
        self
    }
}
