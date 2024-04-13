use std::fmt::Debug;
use std::sync::Arc;

use num_traits::Num;

use crate::types::{Color, Float, Point};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F, T>
where
    Self: Debug + Send + Sync,
    F: Num,
    T: Texel,
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> T;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (u32, u32);

    fn dynsampler(self) -> DynSampler<F, T>
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool;
}

pub type BoxSampler<F, T> = Box<dyn Sampler<F, T>>;
pub type DynSampler<F, T> = Arc<dyn Sampler<F, T>>;

impl<F: Num, T: Texel> Sampler<F, T> for Arc<dyn Sampler<F, T> + 'static> {
    fn sample(&self, uv: Point<F>) -> T {
        (**self).sample(uv)
    }

    fn dimensions(&self) -> (u32, u32) {
        (**self).dimensions()
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        if let Some(samp) = Arc::get_mut(self) {
            samp.ui(ui, name)
        } else {
            ui.label("nope :(");
            false
        }
    }
}

pub trait Texel: Debug + Send + Sync {}

impl Texel for f32 {}
impl Texel for f64 {}

/**
Blanket implementation: [`Sync`] + [`Copy`] types can sample themselves, returning
self as their value.

This is useful to make e.g. a [`crate::Float`] or [`crate::Color<F>`] a viable substitute for a real
texture sampler.
*/
impl<F: Float + Texel> Sampler<F, F> for F
where
    Self: Debug,
{
    fn sample(&self, _uv: Point<F>) -> F {
        *self
    }

    fn dimensions(&self) -> (u32, u32) {
        (1, 1)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        ui.label(name);
        let res = ui
            .add(egui::Slider::new(self, F::zero()..=F::from_u32(128)).clamp_to_range(false))
            .changed();
        ui.end_row();
        res
    }
}

impl<F: Float> Sampler<F, Self> for Color<F> {
    fn sample(&self, _uv: Point<F>) -> Self {
        *self
    }

    fn dimensions(&self) -> (u32, u32) {
        (1, 1)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        crate::gui::controls::color(ui, self, name)
    }
}

pub mod bilinear;
pub mod chessboard;
pub mod heightnormal;
pub mod nearest;
pub mod normalmap;
pub mod perlin;
pub mod samplerext;
pub mod shinemap;
pub mod texture;
pub mod transform;

pub use bilinear::Bilinear;
pub use chessboard::ChessBoardSampler;
pub use heightnormal::HeightNormal;
pub use nearest::Nearest;
pub use normalmap::NormalMap;
pub use perlin::Perlin;
pub use samplerext::SamplerExt;
pub use shinemap::ShineMap;
pub use transform::Adjust;
