use crate::lib::Float;
use crate::lib::ray::{Ray, Hit};
use crate::lib::Vector;
use crate::lib::vector::Vectorx;
use crate::vec3;

use cgmath::Matrix4;

use rtbvh::Primitive;
use rtbvh::Aabb;
use glam::Vec3;

pub trait Geometry<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>;
}

pub trait FiniteGeometry<F: Float> : Geometry<F> + rtbvh::Primitive {}

impl<F: Float> Geometry<F> for Box<dyn Geometry<F>>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        self.as_ref().intersect(ray)
    }
}

impl<F: Float> Geometry<F> for Box<dyn FiniteGeometry<F>>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        self.as_ref().intersect(ray)
    }
}

impl<F: Float> Primitive for Box<dyn FiniteGeometry<F>>
{
    fn center(&self) -> Vec3 {
        self.as_ref().center()
    }

    fn aabb(&self) -> Aabb {
        self.as_ref().aabb()
    }
}

impl<F: Float, G> FiniteGeometry<F> for G
where
    G: Geometry<F> + Send,
    Self: rtbvh::Primitive
{
}

pub fn build_aabb_ranged<F: Float>(xfrm: &Matrix4<F>, x: [F; 2], y: [F; 2], z: [F; 2]) -> Aabb
{
    /* Transform all corner points, expand aabb with each result */
    let mut aabb = Aabb::empty();
    for px in x {
        for py in y {
            for pz in z {
                let p = vec3!(px, py, pz).xfrm(xfrm).into_f32();
                aabb.grow(p.into_vector3());
            }
        }
    }
    aabb
}

pub fn build_aabb_symmetric<F: Float>(xfrm: &Matrix4<F>, x: F, y: F, z: F) -> Aabb
{
    build_aabb_ranged(xfrm, [-x, x], [-y, y], [-z, z])
}

macro_rules! aabb_impl_fm {
    ( $t:ty ) =>
    {
        impl<F: Float, M: Material<F=F>> Primitive for $t
        {
            fn center(&self) -> Vec3 {
                self.aabb.center()
            }

            fn aabb(&self) -> Aabb {
                self.aabb
            }
        }
    }
}

pub(crate) mod geo_util {
    pub use crate::{vec3, point};
    pub use crate::lib::{Vector, Float, Point};
    pub use crate::lib::ray::{Ray, Hit, Maxel};
    pub use crate::lib::vector::{Vectorx, InnerSpace, MetricSpace};
    pub use crate::scene::*;
    pub use crate::material::Material;
    pub use super::Geometry;
    pub use crate::geometry::{build_aabb_ranged, build_aabb_symmetric};

    pub use cgmath::{Matrix4, Transform, Matrix, SquareMatrix};

    pub use num_traits::Zero;

    pub use rtbvh::Primitive;
    pub use rtbvh::Aabb;
    pub use glam::Vec3;
}

pub mod sphere;
pub mod plane;
pub mod triangle;
pub mod trianglemesh;
pub mod cylinder;
pub mod cone;
pub mod cube;
pub mod square;

pub use sphere::Sphere;
pub use plane::Plane;
pub use triangle::Triangle;
pub use trianglemesh::TriangleMesh;
pub use cylinder::Cylinder;
pub use cone::Cone;
pub use cube::Cube;
pub use square::Square;
