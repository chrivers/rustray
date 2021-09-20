use std::marker::PhantomData;

use crate::lib::float::Blended;
use super::samp_util::*;

#[derive(Copy, Clone)]

pub struct Bilinear<F: Float, P: Sync, S: Sampler<F, P>>
{
    samp: S,
    _p0: PhantomData<F>,
    _p1: PhantomData<P>,
}

impl<F: Float, P: Sync, S: Sampler<F, P>> Bilinear<F, P, S>
{
    pub fn new(samp: S) -> Self
    {
        Self { samp, _p0: PhantomData {}, _p1: PhantomData {} }
    }
}

impl<F, P, S> Sampler<F, P> for Bilinear<F, P, S>
where
    F: Float,
    P: Sync,
    P: std::ops::Mul<F, Output = P>,
    P: std::ops::Add<Output = P>,
    P: Blended<F>,
    S: Sampler<F, P>
{
    fn sample(&self, uv: Point<F>) -> P
    {
        let (w, h) = self.dimensions();
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % (w-1);
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % (h-1);
        let fx = (uv.x.abs() * F::from_u32(w)).fract();
        let fy = (uv.y.abs() * F::from_u32(h)).fract();

        let n1 = self.raw_sample(point!(x,   y  ));
        let n2 = self.raw_sample(point!(x+1, y  ));
        let n3 = self.raw_sample(point!(x,   y+1));
        let n4 = self.raw_sample(point!(x+1, y+1));

        let x1 = n1.blended(&n2, fx);
        let x2 = n3.blended(&n4, fx);

        x1.blended(&x2, fy)
    }

    fn raw_sample(&self, uv: Point<u32>) -> P
    {
        self.samp.raw_sample(uv)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }
}
