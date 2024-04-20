use std::fmt::Debug;

use glam::f32::Vec3;
use rtbvh::Aabb;

use crate::material::HasMaterial;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, Maxel, Point, Ray, Transform, Vector, Vectorx};
use crate::vec3;

pub trait Geometry<F: Float>: SceneObject<F> + Debug + Sync + Send {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>;
    fn normal(&self, _maxel: &mut Maxel<F>) -> Vector<F> {
        Vector::ZERO
    }
    fn uv(&self, _maxel: &mut Maxel<F>) -> Point<F> {
        Point::ZERO
    }
    fn st(&self, _maxel: &mut Maxel<F>) -> Point<F> {
        Point::ZERO
    }
    fn material(&mut self) -> Option<&mut dyn HasMaterial>;
}

pub trait FiniteGeometry<F: Float>: Geometry<F> + SceneObject<F> + rtbvh::Primitive {
    fn recompute_aabb(&mut self);
}

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
    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        (**self).material()
    }
}

impl<F: Float> SceneObject<F> for Box<(dyn FiniteGeometry<F> + 'static)> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_icon(&self) -> &str {
        (**self).get_icon()
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

    fn get_icon(&self) -> &str {
        (**self).get_icon()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        (**self).get_interactive()
    }

    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}

impl<F: Float> rtbvh::Primitive for Box<dyn FiniteGeometry<F> + 'static> {
    fn center(&self) -> Vec3 {
        (**self).center()
    }

    fn aabb(&self) -> Aabb {
        (**self).aabb()
    }
}

impl<F: Float> FiniteGeometry<F> for Box<dyn FiniteGeometry<F> + 'static> {
    fn recompute_aabb(&mut self) {
        (**self).recompute_aabb();
    }
}

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
        impl<F: Float> rtbvh::Primitive for $t {
            fn center(&self) -> Vec3 {
                self.aabb.center()
            }

            fn aabb(&self) -> Aabb {
                self.aabb
            }
        }
    };
}

macro_rules! geometry_impl_sceneobject {
    ( $type:ty, $name:expr ) => {
        impl<F: Float> SceneObject<F> for $type {
            crate::sceneobject_impl_body!($name, Self::ICON);
        }
    };
}

macro_rules! geometry_impl_hastransform {
    ( $type:ty ) => {
        impl<F: Float> HasTransform<F> for $type {
            fn get_transform(&self) -> &Transform<F> {
                &self.xfrm
            }

            fn set_transform(&mut self, xfrm: &Transform<F>) {
                self.xfrm = *xfrm;
                self.recompute_aabb();
            }
        }
    };
}

macro_rules! geometry_impl_hasmaterial {
    ( $type:ty ) => {
        impl<F: Float> HasMaterial for $type {
            fn get_material(&self) -> MaterialId {
                self.mat
            }

            fn set_material(&mut self, id: MaterialId) {
                self.mat = id;
            }
        }
    };
}

mod cone;
mod cube;
mod cylinder;
mod plane;
mod sphere;
mod square;
mod triangle;
mod trianglemesh;

pub use cone::Cone;
pub use cube::Cube;
pub use cylinder::Cylinder;
pub use plane::Plane;
pub use sphere::Sphere;
pub use square::Square;
pub use triangle::Triangle;
pub use trianglemesh::TriangleMesh;
