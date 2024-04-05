use std::fmt::Debug;
use std::sync::Arc;

use crate::light::Lixel;
use crate::types::{Color, Float, Maxel};

use crate::scene::RayTracer;

pub trait Material<F: Float>: Debug + Send + Sync {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F>;

    fn shadow(&self, _maxel: &mut Maxel<F>, _lixel: &Lixel<F>) -> Option<Color<F>> {
        Some(Color::BLACK)
    }

    fn dynamic(self) -> DynMaterial<F>
    where
        Self: Sized + 'static,
    {
        Arc::new(Box::new(self))
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Unknown material");
    }
}

pub type DynMaterial<F> = Arc<Box<dyn Material<F>>>;

impl<F: Float> Material<F> for Color<F> {
    fn render(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Self {
        *self
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        crate::frontend::gui::color_ui(ui, self, "Color");
    }
}

impl<F: Float> Material<F> for Arc<Box<dyn Material<F>>> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        self.as_ref().render(maxel, rt)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        match Arc::<Box<_>>::get_mut(self) {
            Some(mat) => mat.ui(ui),
            None => {
                ui.label("nope :(");
            }
        }
    }
}

pub(crate) mod mat_util {
    /* These are convenience re-imports for modules, so skip warnings */
    #![allow(unused_imports)]
    pub use crate::light::{Light, Lixel};
    pub use crate::material::{DynMaterial, Material};
    pub use crate::sampler::Sampler;
    pub use crate::sampler::Texel;
    pub use crate::scene::{Interactive, RayTracer};
    pub use crate::types::{Color, Float, Maxel, Point, Ray, Vector, Vectorx};
    pub use crate::{point, vec3};

    #[cfg(feature = "gui")]
    pub use crate::frontend::gui::color_ui;
    #[cfg(feature = "gui")]
    pub use egui::{CollapsingHeader, Slider};

    pub use cgmath::{InnerSpace, VectorSpace};

    use num_traits::Zero;
    pub use std::marker::PhantomData;
}

mod blend;
mod bumpmap;
mod chessboard;
mod chessboardxyz;
mod colornormal;
mod colorpos;
mod colorst;
mod coloruv;
mod fresnel;
mod matte;
mod mirror;
mod phong;
mod scaleuv;
mod smart;
mod texture;
mod triblend;

pub use blend::Blend;
pub use bumpmap::Bumpmap;
pub use chessboard::ChessBoard;
pub use chessboardxyz::ChessBoardXYZ;
pub use colornormal::ColorNormal;
pub use colorpos::ColorPos;
pub use colorst::ColorST;
pub use coloruv::ColorUV;
pub use fresnel::Fresnel;
pub use matte::Matte;
pub use mirror::Mirror;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use smart::Smart;
pub use texture::{Texture, TextureSampler};
pub use triblend::Triblend;
