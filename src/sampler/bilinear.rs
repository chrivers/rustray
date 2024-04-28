use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::point;
use crate::sampler::{Sampler, Texel};
use crate::types::{Float, Point};

#[derive(Copy, Clone)]
pub struct Bilinear<P: Texel, S: Sampler<u32, P>> {
    samp: S,
    _p0: PhantomData<P>,
}

impl<P: Texel, S: Sampler<u32, P>> Bilinear<P, S> {
    pub const fn new(samp: S) -> Self {
        Self {
            samp,
            _p0: PhantomData {},
        }
    }
}

impl<P: Texel, S: Sampler<u32, P>> Debug for Bilinear<P, S> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Bilinear")
            .field("<type>", &self._p0)
            .finish_non_exhaustive()
    }
}

impl<F, P, S> Sampler<F, P> for Bilinear<P, S>
where
    F: Float,
    P: Texel<Ratio = F>,
    S: Sampler<u32, P>,
{
    fn sample(&self, uv: Point<F>) -> P {
        let (w, h) = self.dimensions();

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

        /* Fractional coordinate part */
        let fx = rx.fract();
        let fy = ry.fract();

        let n1 = self.samp.sample(point!(x, y));
        let n2 = self.samp.sample(point!(x + 1, y));
        let n3 = self.samp.sample(point!(x, y + 1));
        let n4 = self.samp.sample(point!(x + 1, y + 1));

        let x1 = n1.lerp(n2, fx);
        let x2 = n3.lerp(n4, fx);

        x1.lerp(x2, fy)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        self.samp.ui(ui, &format!("{name} (bilinear)"))
    }
}
