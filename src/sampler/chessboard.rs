use std::marker::PhantomData;

use crate::sampler::{Sampler, Texel};
use crate::types::{Float, Point};

#[derive(Copy, Clone, Debug)]
pub struct ChessBoardSampler<F: Float, T: Texel, A: Sampler<F, T>, B: Sampler<F, T>> {
    a: A,
    b: B,
    f: PhantomData<F>,
    t: PhantomData<T>,
}

impl<F: Float, T: Texel, A: Sampler<F, T>, B: Sampler<F, T>> ChessBoardSampler<F, T, A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            f: PhantomData,
            t: PhantomData,
        }
    }
}

impl<F, T, A, B> Sampler<F, T> for ChessBoardSampler<F, T, A, B>
where
    F: Float,
    T: Texel,
    A: Sampler<F, T>,
    B: Sampler<F, T>,
{
    fn sample(&self, uv: Point<F>) -> T {
        let u = uv.x.abs().fract() > F::HALF;
        let v = uv.y.abs().fract() > F::HALF;

        if u ^ v {
            self.a.sample(uv)
        } else {
            self.b.sample(uv)
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        self.a.dimensions()
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        ui.strong(name);
        ui.end_row();
        let mut res = false;
        res |= self.a.ui(ui, name);
        res |= self.b.ui(ui, name);
        res
    }
}
