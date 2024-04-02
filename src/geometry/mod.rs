use std::fmt::Debug;

use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, Maxel, Point, Ray, Transform, Vector, Vectorx};
use crate::vec3;

use num_traits::Zero;

use rtbvh::Aabb;

use glam::f32::Vec3;

pub trait Geometry<F: Float>: SceneObject<F> + Debug + Sync + Send {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>;
    fn normal(&self, _maxel: &mut Maxel<F>) -> Vector<F> {
        Vector::zero()
    }
    fn uv(&self, _maxel: &mut Maxel<F>) -> Point<F> {
        Point::zero()
    }
    fn st(&self, _maxel: &mut Maxel<F>) -> Point<F> {
        Point::zero()
    }
}

pub trait FiniteGeometry<F: Float>: Geometry<F> + SceneObject<F> + rtbvh::Primitive {}

impl<F: Float, T> Geometry<F> for Box<T>
where
    T: Geometry<F> + ?Sized,
    Self: SceneObject<F>,
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        (**self).intersect(ray)
    }

    fn normal(&self, maxel: &mut Maxel<F>) -> Vector<F> {
        (**self).normal(maxel)
    }

    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F> {
        (**self).uv(maxel)
    }

    fn st(&self, maxel: &mut Maxel<F>) -> Point<F> {
        (**self).st(maxel)
    }
}

impl<F: Float> SceneObject<F> for Box<(dyn FiniteGeometry<F> + 'static)> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        (**self).get_interactive()
    }

    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}

impl<F: Float> SceneObject<F> for Box<(dyn Geometry<F> + 'static)> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        (**self).get_interactive()
    }
    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}

impl<'a, F: Float> rtbvh::Primitive for Box<dyn FiniteGeometry<F> + 'a> {
    fn center(&self) -> Vec3 {
        (**self).center()
    }

    fn aabb(&self) -> Aabb {
        (**self).aabb()
    }
}

impl<F: Float, G: Geometry<F> + rtbvh::Primitive> FiniteGeometry<F> for G {}

pub fn build_aabb_ranged<F: Float>(xfrm: &Transform<F>, x: [F; 2], y: [F; 2], z: [F; 2]) -> Aabb {
    /* Transform all corner points, expand aabb with each result */
    let mut aabb = Aabb::empty();
    for px in x {
        for py in y {
            for pz in z {
                let p = xfrm.pos(vec3!(px, py, pz)).into_f32();
                aabb.grow(p.into_vec3());
            }
        }
    }
    aabb
}

pub fn build_aabb_symmetric<F: Float>(xfrm: &Transform<F>, x: F, y: F, z: F) -> Aabb {
    build_aabb_ranged(xfrm, [-x, x], [-y, y], [-z, z])
}

macro_rules! aabb_impl_fm {
    ( $t:ty ) => {
        impl<F: Float, M: Material<F>> rtbvh::Primitive for $t {
            fn center(&self) -> Vec3 {
                self.aabb.center()
            }

            fn aabb(&self) -> Aabb {
                self.aabb
            }
        }
    };
}

pub(crate) mod geo_util {
    pub use super::Geometry;
    pub use crate::geometry::{build_aabb_ranged, build_aabb_symmetric};
    pub use crate::material::Material;
    pub use crate::scene::{Interactive, SceneObject};
    pub use crate::types::transform::{HasTransform, Transform};
    pub use crate::types::{Float, Maxel, Point, Ray, Vector, Vectorx};
    pub use crate::{point, vec3};

    #[cfg(feature = "gui")]
    pub use crate::frontend::gui::position_ui;
    #[cfg(feature = "gui")]
    pub use egui::Slider;

    pub use cgmath::{InnerSpace, Matrix4};

    pub use num_traits::Zero;

    pub use glam::Vec3;
    pub use rtbvh::Aabb;
    pub use rtbvh::Primitive;
}

pub mod cone;
pub mod cube;
pub mod cylinder;
pub mod plane;
pub mod sphere;
pub mod square;
pub mod triangle;
pub mod trianglemesh;

pub use cone::Cone;
pub use cube::Cube;
pub use cylinder::Cylinder;
pub use plane::Plane;
pub use sphere::Sphere;
pub use square::Square;
pub use triangle::Triangle;
pub use trianglemesh::TriangleMesh;
