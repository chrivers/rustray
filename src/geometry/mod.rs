use crate::lib::Float;
use crate::lib::ray::{Ray, Hit};

pub trait Geometry<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>;
}

pub(crate) mod geo_util {
    pub use crate::{vec3, point};
    pub use crate::lib::{Vector, Float, Point};
    pub use crate::lib::ray::{Ray, Hit, Maxel};
    pub use crate::lib::vector::{Vectorx, InnerSpace, MetricSpace};
    pub use crate::scene::*;
    pub use crate::material::Material;
    pub use super::Geometry;

    pub use cgmath::{Matrix4, Transform};
}

pub mod sphere;
pub mod plane;
pub mod triangle;
pub mod trianglemesh;
pub mod cylinder;
pub mod cone;

pub use sphere::Sphere;
pub use plane::Plane;
pub use triangle::Triangle;
pub use trianglemesh::TriangleMesh;
pub use cylinder::Cylinder;
pub use cone::Cone;
