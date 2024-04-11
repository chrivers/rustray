use std::fmt::Debug;
use std::sync::Arc;

use crate::light::Lixel;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::matlib::MaterialId;
use crate::types::{Color, Float, Maxel};

pub trait Material<F: Float>: SceneObject<F> + Interactive<F> + Debug + Send + Sync {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F>;

    fn shadow(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>, _lixel: &Lixel<F>) -> Color<F> {
        Color::BLACK
    }

    fn dynamic(self) -> DynMaterial<F>
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }
}

pub type BoxMaterial<F> = Box<dyn Material<F>>;
pub type DynMaterial<F> = Arc<dyn Material<F>>;

impl<F: Float> Material<F> for Color<F> {
    fn render(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Self {
        *self
    }
}

#[cfg(feature = "gui")]
impl<F: Float> SceneObject<F> for Box<dyn Material<F>> {
    sceneobject_impl_body!("Material");
}

impl<F: Float> SceneObject<F> for Color<F> {
    sceneobject_impl_body!("Color");
}

impl<F: Float> SceneObject<F> for MaterialId {
    sceneobject_impl_body!("Material ID");
}

impl<F: Float> Material<F> for MaterialId {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let mat = &rt.scene().materials.mats[self];
        mat.render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        let mat = &rt.scene().materials.mats[self];
        mat.shadow(maxel, rt, lixel)
    }
}

impl<F: Float> Material<F> for Box<dyn Material<F>>
where
    Self: Interactive<F>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        (**self).render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        (**self).shadow(maxel, rt, lixel)
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Box<dyn Material<F>> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        (**self).ui(ui)
    }
}

impl<F: Float> Material<F> for Arc<dyn Material<F>> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        (**self).render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        (**self).shadow(maxel, rt, lixel)
    }
}

#[cfg(feature = "gui")]
impl<F: Float> SceneObject<F> for Arc<dyn Material<F>> {
    sceneobject_impl_body!("Material");
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Arc<dyn Material<F>> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        if let Some(mat) = Arc::get_mut(self) {
            mat.ui(ui)
        } else {
            ui.label("nope :(");
            false
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
    pub use crate::scene::{Interactive, RayTracer, SceneObject};
    pub use crate::types::{Color, Float, Maxel, Point, Ray, Vector, Vectorx};
    pub use crate::{point, sceneobject_impl_body, vec3};

    #[cfg(feature = "gui")]
    pub use crate::frontend::gui::controls;
    #[cfg(feature = "gui")]
    pub use egui::Slider;

    pub use cgmath::{InnerSpace, VectorSpace, Zero};

    pub use std::marker::PhantomData;
}

mod blend;
mod bumpmap;
mod chessboard;
mod debug;
mod fresnel;
mod matte;
mod mirror;
mod phong;
mod scaleuv;
mod smart;
mod texture;
mod triblend;

pub use blend::Blend;
pub use bumpmap::{BumpPower, Bumpmap};
pub use chessboard::{ChessBoard, ChessBoardMode};
pub use debug::ColorDebug;
pub use fresnel::Fresnel;
pub use matte::Matte;
pub use mirror::Mirror;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use smart::Smart;
pub use texture::{Texture, TextureSampler};
pub use triblend::Triblend;
