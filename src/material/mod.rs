use crate::math::{Vector, Float, Point, Color};
use crate::math::ray::{Ray, Hit, Maxel};

use crate::light::Light;
use crate::scene::RayTracer;

pub trait Material : Sync
{
    type F: Float;
    fn render(&self, hit: &Hit<Self::F>, maxel: &Maxel<Self::F>, light: &[Light<Self::F>], rt: &dyn RayTracer<Self::F>, lvl: u32) -> Color<Self::F>;
}


impl<F: Float> Material for Color<F>
{
    type F = F;
    fn render(&self, _hit: &Hit<F>, _maxel: &Maxel<F>, _light: &[Light<F>], _rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        *self
    }
}

pub(crate) mod mat_util {
    pub use crate::math::{Vector, Float, Point, Color};
    pub use crate::math::ray::{Ray, Hit, Maxel};
    pub use crate::light::Light;
    pub use crate::scene::RayTracer;
    pub use crate::vec3;

    pub use super::Material;
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

pub use chessboard::ChessBoard;
pub use chessboardxyz::ChessBoardXYZ;
pub use mirror::Mirror;
pub use fresnel::Fresnel;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use blend::Blend;
pub use texture::Texture;
pub use bumpmap::Bumpmap;
pub use colornormal::ColorNormal;
