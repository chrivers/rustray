use std::sync::Arc;
use std::fmt::Debug;

use crate::types::{Float, Color};
use crate::types::ray::Maxel;

use crate::scene::{RayTracer, Light};

pub trait Material : Debug + Send + Sync
{
    type F: Float;
    fn render(&self, maxel: &mut Maxel<Self::F>, light: &[&dyn Light<Self::F>], rt: &dyn RayTracer<Self::F>) -> Color<Self::F>;

    fn shadow(&self, _maxel: &mut Maxel<Self::F>, _light: &dyn Light<Self::F>) -> Option<Color<Self::F>> {
        Some(Color::<Self::F>::black())
    }

    fn dynamic<'a>(self) -> DynMaterial<'a, Self::F>
    where
        Self: Sized + 'a
    {
        Arc::new(Box::new(self))
    }
}

pub type DynMaterial<'a, F> = Arc<Box<dyn Material<F=F> + 'a>>;

impl<F: Float> Material for Color<F>
{
    type F = F;
    fn render(&self, _maxel: &mut Maxel<F>, _light: &[&dyn Light<F>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        *self
    }
}

impl<'a, F: Float> Material for Arc<Box<dyn Material<F=F> + 'a>>
{
    type F = F;
    fn render(&self, maxel: &mut Maxel<F>, light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        self.as_ref().render(maxel, light, rt)
    }
}

pub(crate) mod mat_util {
    /* These are convenience re-imports for modules, so skip warnings */
    #![allow(unused_imports)]
    pub use crate::types::{Vector, Float, Point, Color};
    pub use crate::types::ray::{Ray, Maxel};
    pub use crate::types::vector::{Vectorx, InnerSpace};
    pub use crate::scene::{RayTracer, Light};
    pub use crate::sampler::Sampler;
    pub use crate::{vec3, point};
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
