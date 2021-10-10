use std::sync::Arc;

use crate::lib::{Float, Color};
use crate::lib::ray::{Hit, Maxel};

use crate::scene::{RayTracer, Light};

pub trait Material : Sync
{
    type F: Float;
    fn render(&self, hit: &Hit<Self::F>, maxel: &Maxel<Self::F>, light: &[Box<dyn Light<Self::F>>], rt: &dyn RayTracer<Self::F>) -> Color<Self::F>;

    fn shadow(&self, _hit: &Hit<Self::F>, _maxel: &Maxel<Self::F>, _light: &dyn Light<Self::F>, _rt: &dyn RayTracer<Self::F>) -> Option<Color<Self::F>> {
        Some(Color::<Self::F>::black())
    }
}


impl<F: Float> Material for Color<F>
{
    type F = F;
    fn render(&self, _hit: &Hit<F>, _maxel: &Maxel<F>, _light: &[Box<dyn Light<F>>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        *self
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
    pub use cgmath::VectorSpace;

    pub use super::Material;
    pub use std::marker::PhantomData;
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
mod coloruv;
mod matte;

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
pub use coloruv::ColorUV;
pub use matte::Matte;
