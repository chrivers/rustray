use crate::lib::Float;
use crate::lib::ray::{Ray, Hit};
use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use crate::lib::Vector;
use crate::lib::vector::Vectorx;
use crate::vec3;
use crate::lib::point::Point;
use cgmath::Matrix4;

pub trait Geometry<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>;
}

pub trait FiniteGeometry<F: Float> : Geometry<F> + BHShape {}

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

impl<F: Float> Bounded for Box<dyn FiniteGeometry<F>>
{
    fn aabb(&self) -> AABB {
        self.as_ref().aabb()
    }
}

impl<F: Float> BHShape for Box<dyn FiniteGeometry<F>>
{
    fn set_bh_node_index(&mut self, index: usize) {
        self.as_mut().set_bh_node_index(index)
    }

    fn bh_node_index(&self) -> usize {
        self.as_ref().bh_node_index()
    }
}

impl<F: Float, G> FiniteGeometry<F> for G
where
    G: Geometry<F>,
    Self: BHShape + Bounded
{
}

pub fn build_aabb_ranged<F: Float>(xfrm: &Matrix4<F>, x: [F; 2], y: [F; 2], z: [F; 2]) -> AABB
{
    /* Transform all corner points, expand aabb with each result */
    let mut aabb: AABB = AABB::empty();
    for px in x {
        for py in y {
            for pz in z {
                let p = vec3!(px, py, pz).xfrm(&xfrm);
                aabb.grow_mut(&p.into_point3());
            }
        }
    }
    aabb
}

pub fn build_aabb_symmetric<F: Float>(xfrm: &Matrix4<F>, x: F, y: F, z: F) -> AABB
{
    build_aabb_ranged(xfrm, [-x, x], [-y, y], [-z, z])
}

macro_rules! aabb_impl_fm {
    ( $t:ty ) =>
    {
        impl<F: Float, M: Material<F=F>> Bounded for $t
        {
            fn aabb(&self) -> AABB { self.aabb }
        }

        impl<F: Float, M: Material<F=F>> BHShape for $t
        {
            fn set_bh_node_index(&mut self, index: usize)
            {
                self.ni = index;
            }

            fn bh_node_index(&self) -> usize
            {
                self.ni
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

    pub use cgmath::{Matrix4, Transform, SquareMatrix};

    pub use bvh::aabb::{AABB, Bounded};
    pub use bvh::bounding_hierarchy::BHShape;
    pub use bvh::Point3;
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
