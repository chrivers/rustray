use super::samp_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Adjust<F: Float, T: Texel, S: Sampler<F, T>>
{
    s: F, // scale
    o: F, // offset
    samp: S,
    _p1: PhantomData<T>,
}

impl<F: Float, T: Texel, S: Sampler<F, T>> Adjust<F, T, S>
{
    pub fn new(s: F, o: F, samp: S) -> Self
    {
        Self { s, o, samp, _p1: PhantomData {} }
    }
}

impl<F, T, S> Sampler<F, T> for Adjust<F, T, S>
where
    F: Float,
    T: Texel,
    T: std::ops::Mul<F, Output = T>,
    T: std::ops::Add<F, Output = T>,
    T: Lerp<Ratio=F>,
    S: Sampler<F, T>
{
    fn sample(&self, uv: Point<F>) -> T
    {
        let s = self.samp.sample(uv);
        s * self.s + self.o
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }
}
