use std::sync::Arc;
use std::fmt::Debug;

use crate::lib::{Float, Color};
use crate::lib::ray::{Hit, Maxel};

use crate::scene::{RayTracer, Light};

pub trait Material : Debug + Send + Sync
{
    type F: Float;
    fn render(&self, hit: &Hit<Self::F>, maxel: &Maxel<Self::F>, light: &[&dyn Light<Self::F>], rt: &dyn RayTracer<Self::F>) -> Color<Self::F>;

    fn shadow(&self, _hit: &Hit<Self::F>, _maxel: &Maxel<Self::F>, _light: &dyn Light<Self::F>) -> Option<Color<Self::F>> {
        Some(Color::<Self::F>::black())
    }

    fn dynamic<'a>(self) -> DynMaterial<'a, Self::F>
    where
        Self: Sized + 'a
    {
        Arc::new(Box::new(self) as Box<dyn Material<F=Self::F> + 'a>)
    }
}

pub type DynMaterial<'a, F> = Arc<Box<dyn Material<F=F> + 'a>>;

impl<F: Float> Material for Color<F>
{
    type F = F;
    fn render(&self, _hit: &Hit<F>, _maxel: &Maxel<F>, _light: &[&dyn Light<F>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        *self
    }
}

impl<F: Float> Material for Arc<Box<dyn Material<F=F>>>
{
    type F = F;
    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        self.as_ref().render(hit, maxel, light, rt)
    }
}

pub(crate) mod mat_util {
    /* These are convenience re-imports for modules, so skip warnings */
    #![allow(unused_imports)]
    pub use crate::lib::{Vector, Float, Point, Color};
    pub use crate::lib::ray::{Ray, Hit, Maxel};
    pub use crate::lib::vector::{Vectorx, InnerSpace};
    pub use crate::scene::{RayTracer, Light};
    pub use crate::sampler::Sampler;
    pub use crate::vec3;
    pub use crate::material::{Material, DynMaterial};
    pub use crate::sampler::Texel;

    pub use cgmath::VectorSpace;

    pub use std::marker::PhantomData;
    use num_traits::Zero;
}

mod chessboard;
mod chessboardxyz;
mod mirror;
mod fresnel;
mod phong;
mod scaleuv;
mod blend;
mod texture;
mod bumpmap;
mod colornormal;
mod colorpos;
mod colorst;
mod coloruv;
mod matte;
mod smart;
mod triblend;

pub use chessboard::ChessBoard;
pub use chessboardxyz::ChessBoardXYZ;
pub use mirror::Mirror;
pub use fresnel::Fresnel;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use blend::Blend;
pub use texture::{Texture, TextureSampler};
pub use bumpmap::Bumpmap;
pub use colornormal::ColorNormal;
pub use colorpos::ColorPos;
pub use colorst::ColorST;
pub use coloruv::ColorUV;
pub use matte::Matte;
pub use smart::Smart;
pub use triblend::Triblend;
