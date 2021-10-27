use crate::lib::{Float, Point};
use crate::lib::ray::{Ray, Maxel};
use crate::lib::Vector;
use crate::lib::vector::Vectorx;
use crate::vec3;
use crate::lib::transform::Transform;

use num_traits::Zero;

use rtbvh::Primitive;
use rtbvh::Aabb;
use glam::Vec3;

pub trait Geometry<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>;
    fn normal(&self, _maxel: &mut Maxel<F>) -> Vector<F> { Vector::zero() }
    fn uv(&self, _maxel: &mut Maxel<F>) -> Point<F> { Point::zero() }
    fn st(&self, _maxel: &mut Maxel<F>) -> Point<F> { Point::zero() }
}

pub trait FiniteGeometry<F: Float> : Geometry<F> + rtbvh::Primitive {}

impl<F: Float> Geometry<F> for Box<dyn Geometry<F>>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        self.as_ref().intersect(ray)
    }

    fn normal(&self, maxel: &mut Maxel<F>) -> Vector<F>
    {
        self.as_ref().normal(maxel)
    }

    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F>
    {
        self.as_ref().uv(maxel)
    }

    fn st(&self, maxel: &mut Maxel<F>) -> Point<F>
    {
        self.as_ref().st(maxel)
    }
}

impl<F: Float> Geometry<F> for Box<dyn FiniteGeometry<F>>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        self.as_ref().intersect(ray)
    }

    fn normal(&self, maxel: &mut Maxel<F>) -> Vector<F>
    {
        self.as_ref().normal(maxel)
    }

    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F>
    {
        self.as_ref().uv(maxel)
    }

    fn st(&self, maxel: &mut Maxel<F>) -> Point<F>
    {
        self.as_ref().st(maxel)
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

pub fn build_aabb_ranged<F: Float>(xfrm: &Transform<F>, x: [F; 2], y: [F; 2], z: [F; 2]) -> Aabb
{
    /* Transform all corner points, expand aabb with each result */
    let mut aabb = Aabb::empty();
    for px in x {
        for py in y {
            for pz in z {
                let p = xfrm.pos(vec3!(px, py, pz)).into_f32();
                aabb.grow(p.into_vector3());
            }
        }
    }
    aabb
}

pub fn build_aabb_symmetric<F: Float>(xfrm: &Transform<F>, x: F, y: F, z: F) -> Aabb
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
    pub use crate::lib::ray::{Ray, Maxel};
    pub use crate::lib::vector::{Vectorx, InnerSpace, MetricSpace};
    pub use crate::lib::transform::Transform;
    pub use crate::scene::*;
    pub use crate::material::Material;
    pub use super::Geometry;
    pub use crate::geometry::{build_aabb_ranged, build_aabb_symmetric};

    pub use cgmath::{Matrix4, Transform as cgTransform, Matrix, SquareMatrix};

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
