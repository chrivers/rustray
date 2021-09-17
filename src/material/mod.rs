use crate::ray::Maxel;
use crate::color::Color;
use crate::Float;
use crate::light::Light;
use crate::scene::RayTracer;
use crate::ray::Hit;

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
    pub use crate::color::Color;
    pub use crate::ray::{Ray, Hit, Maxel};
    pub use crate::light::Light;
    pub use crate::scene::RayTracer;
    pub use crate::traits::Float;
    pub use crate::vector::Vector;
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

pub use chessboard::ChessBoard;
pub use chessboardxyz::ChessBoardXYZ;
pub use mirror::Mirror;
pub use fresnel::Fresnel;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use blend::Blend;
