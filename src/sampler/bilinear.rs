use std::marker::PhantomData;

use image::DynamicImage;
use super::samp_util::*;

pub trait BilinearSampler<F: Float, P: Pixel>
where
    Self: Sampler<F, P> + Sized
{
    fn bilinear(self) -> Bilinear<F, P, Self> {
        Bilinear::new(self)
    }
}

impl<F: Float, P: Pixel> BilinearSampler<F, P> for image::DynamicImage
where
    DynamicImage: Sampler<F, P>
{
}

#[derive(Copy, Clone)]

pub struct Bilinear<F: Float, P: Pixel, S: Sampler<F, P>>
{
    samp: S,
    _p0: PhantomData<F>,
    _p1: PhantomData<P>,
}

impl<F: Float, P: Pixel, S: Sampler<F, P>> Bilinear<F, P, S>
{
    pub fn new(samp: S) -> Self
    {
        Self { samp, _p0: PhantomData {}, _p1: PhantomData {} }
    }
}

impl<F, P, S> Sampler<F, P> for Bilinear<F, P, S>
where
    F: Float,
    P: Pixel,
    P: std::ops::Mul<F, Output = P>,
    P: std::ops::Add<Output = P>,
    P: Lerp<Ratio=F>,
    S: Sampler<F, P>
{
    fn sample(&self, uv: Point<F>) -> P
    {
        let (w, h) = self.dimensions();

        /* Raw (x, y) coordinates */
        let rx = (uv.x * F::from_u32(w) - F::ONE / F::from_u32(w/2)).max(F::ZERO).min(F::from_u32(w-1));
        let ry = (uv.y * F::from_u32(h) - F::ONE / F::from_u32(h/2)).max(F::ZERO).min(F::from_u32(h-1));

        /* Integer coordinate part */
        let x  = rx.trunc().to_u32().unwrap_or(0);
        let y  = ry.trunc().to_u32().unwrap_or(0);

        /* Fractional coordinate part */
        let fx = rx.fract();
        let fy = ry.fract();

        let n1 = self.raw_sample(point!(x,   y  ));
        let n2 = self.raw_sample(point!(x+1, y  ));
        let n3 = self.raw_sample(point!(x,   y+1));
        let n4 = self.raw_sample(point!(x+1, y+1));

        let x1 = n1.lerp(n2, fx);
        let x2 = n3.lerp(n4, fx);

        x1.lerp(x2, fy)
    }

    fn raw_sample(&self, uv: Point<u32>) -> P
    {
        self.samp.raw_sample(uv)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }
}
