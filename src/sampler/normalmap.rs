use std::marker::PhantomData;

use cgmath::InnerSpace;

use crate::sampler::Sampler;
use crate::types::{Color, Float, Point, Vector};

#[derive(Copy, Clone, Debug)]
pub struct NormalMap<F: Float, S: Sampler<F, Color<F>>> {
    sampler: S,
    _p: PhantomData<F>,
}

impl<F: Float, S: Sampler<F, Color<F>>> NormalMap<F, S> {
    pub const fn new(sampler: S) -> Self {
        Self {
            sampler,
            _p: PhantomData {},
        }
    }

    pub fn color_to_vector(col: &Color<F>) -> Vector<F> {
        let mut n = *col;
        n.r -= F::HALF;
        n.g -= F::HALF;
        n.r *= F::TWO;
        n.g *= F::TWO;
        Vector::new(n.r, n.g, n.b).normalize()
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> Sampler<F, Vector<F>> for NormalMap<F, S> {
    fn sample(&self, uv: Point<F>) -> Vector<F> {
        Self::color_to_vector(&self.sampler.sample(uv))
    }

    fn dimensions(&self) -> (u32, u32) {
        self.sampler.dimensions()
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        ui.strong("Normal map");
        ui.end_row();
        self.sampler.ui(ui, name)
    }
}
