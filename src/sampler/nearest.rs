use super::samp_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Nearest<P: Texel, S: Sampler<u32, P>> {
    samp: S,
    _p0: PhantomData<P>,
}

impl<P: Texel, S: Sampler<u32, P>> Nearest<P, S> {
    pub const fn new(samp: S) -> Self {
        Self {
            samp,
            _p0: PhantomData {},
        }
    }
}

impl<F, P, S> Sampler<F, P> for Nearest<P, S>
where
    F: Float,
    P: Texel,
    S: Sampler<u32, P>,
{
    fn sample(&self, uv: Point<F>) -> P {
        let (w, h) = self.samp.dimensions();

        /* Raw (x, y) coordinates */
        let rx = (uv.x * F::from_u32(w) - F::ONE / F::from_u32(w / 2))
            .max(F::ZERO)
            .min(F::from_u32(w - 1));
        let ry = (uv.y * F::from_u32(h) - F::ONE / F::from_u32(h / 2))
            .max(F::ZERO)
            .min(F::from_u32(h - 1));

        /* Integer coordinate part */
        let x = rx.trunc().to_u32().unwrap_or(0);
        let y = ry.trunc().to_u32().unwrap_or(0);

        self.samp.sample(point!(x, y))
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }
}
