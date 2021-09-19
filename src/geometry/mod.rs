pub(crate) mod geo_util {
    pub use crate::{vec3, point};
    pub use crate::lib::{Vector, Float, Point};
    pub use crate::lib::ray::{Ray, Hit, Maxel};
    pub use crate::scene::*;
    pub use crate::material::Material;
}

pub mod sphere;
pub mod plane;
pub mod triangle;
pub mod trianglemesh;

pub use sphere::Sphere;
pub use plane::Plane;
pub use triangle::Triangle;
pub use trianglemesh::TriangleMesh;
