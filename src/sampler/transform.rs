use super::samp_util::*;

#[derive(Copy, Clone)]

pub struct Adjust<F: Float, P: Sync, S: Sampler<F, P>>
{
    s: F, // scale
    o: F, // offset
    samp: S,
    _p1: PhantomData<P>,
}

impl<F: Float, P: Sync, S: Sampler<F, P>> Adjust<F, P, S>
{
    pub fn new(s: F, o: F, samp: S) -> Self
    {
        Self { s, o, samp, _p1: PhantomData {} }
    }
}

impl<F, P, S> Sampler<F, P> for Adjust<F, P, S>
where
    F: Float,
    P: Sync,
    P: std::ops::Mul<F, Output = P>,
    P: std::ops::Add<F, Output = P>,
    P: Lerp<Ratio=F>,
    S: Sampler<F, P>
{
    fn sample(&self, uv: Point<F>) -> P
    {
        let s = self.samp.sample(uv);
        s * self.s + self.o
    }

    fn raw_sample(&self, uv: Point<u32>) -> P
    {
        self.samp.raw_sample(uv)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }
}
