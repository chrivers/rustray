use std::fmt::Debug;
use std::sync::Arc;

use crate::types::ray::Maxel;
use crate::types::{Color, Float};

use crate::scene::{Light, RayTracer};

pub trait Material: Debug + Send + Sync {
    type F: Float;
    fn render(
        &self,
        maxel: &mut Maxel<Self::F>,
        light: &[&dyn Light<Self::F>],
        rt: &dyn RayTracer<Self::F>,
    ) -> Color<Self::F>;

    fn shadow(
        &self,
        _maxel: &mut Maxel<Self::F>,
        _light: &dyn Light<Self::F>,
    ) -> Option<Color<Self::F>> {
        Some(Color::black())
    }

    fn dynamic<'a>(self) -> DynMaterial<'a, Self::F>
    where
        Self: Sized + 'a,
    {
        Arc::new(Box::new(self))
    }
}

pub type DynMaterial<'a, F> = Arc<Box<dyn Material<F = F> + 'a>>;

impl<F: Float> Material for Color<F> {
    type F = F;
    fn render(
        &self,
        _maxel: &mut Maxel<F>,
        _light: &[&dyn Light<F>],
        _rt: &dyn RayTracer<F>,
    ) -> Color<F> {
        *self
    }
}

impl<'a, F: Float> Material for Arc<Box<dyn Material<F = F> + 'a>> {
    type F = F;
    fn render(
        &self,
        maxel: &mut Maxel<F>,
        light: &[&dyn Light<F>],
        rt: &dyn RayTracer<F>,
    ) -> Color<F> {
        self.as_ref().render(maxel, light, rt)
    }
}

pub(crate) mod mat_util {
    /* These are convenience re-imports for modules, so skip warnings */
    #![allow(unused_imports)]
    pub use crate::material::{DynMaterial, Material};
    pub use crate::sampler::Sampler;
    pub use crate::sampler::Texel;
    pub use crate::scene::{Light, RayTracer};
    pub use crate::types::ray::{Maxel, Ray};
    pub use crate::types::vector::{InnerSpace, Vectorx};
    pub use crate::types::{Color, Float, Point, Vector};
    pub use crate::{point, vec3};

    pub use cgmath::VectorSpace;

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
