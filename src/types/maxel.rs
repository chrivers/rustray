use core::fmt::{self, Debug};

use crate::geometry::Geometry;
use crate::light::Lixel;
use crate::types::{Float, MaterialId, Point, Ray, RayFlags, Transform, Vector, Vectorx, RF};

#[derive(Copy, Clone)]
pub struct Maxel<'a, F: Float> {
    /// Intersection point in object space
    pub hit: Vector<F>,
    /// Intersection point in world space
    pub pos: Vector<F>,
    /// Intersection direction
    pub dir: Vector<F>,
    /// Intersected object
    pub obj: &'a dyn Geometry<F>,
    /// Material id at intersection
    pub mat: MaterialId,
    /// Normal at intersection
    nml: Option<Vector<F>>,
    /// Texture (u, v) coordinates at intersection
    uv: Option<Point<F>>,
    /// Object (s, t) coordinates at intersection
    st: Option<Point<F>>,
    /// Ray nesting level
    pub lvl: u16,
    /// Ray flags from intersecting ray
    pub flags: RayFlags,
}

impl<'a, F: Float> Debug for Maxel<'a, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Maxel")
            .field("pos", &self.pos)
            .field("dir", &self.dir)
            .field("obj", &self.obj)
            .field("mat", &self.mat)
            .field("lvl", &self.lvl)
            .field("nml", &self.nml)
            .field("uv", &self.uv)
            .field("st", &self.st)
            .finish()
    }
}

/* Maxel */
impl<'a, F: Float> Maxel<'a, F> {
    pub fn new(
        hit: Vector<F>,
        pos: Vector<F>,
        dir: Vector<F>,
        lvl: u16,
        obj: &'a dyn Geometry<F>,
        mat: MaterialId,
        flags: RayFlags,
    ) -> Self {
        Maxel {
            hit,
            pos,
            dir,
            lvl,
            obj,
            mat,
            nml: None,
            uv: None,
            st: None,
            flags,
        }
    }

    #[must_use]
    pub fn xfrm(mut self, xfrm: &Transform<F>) -> Self {
        self.pos = xfrm.pos(self.pos);
        self.dir = xfrm.dir(self.dir);
        self.nml = self.nml.map(|nml| xfrm.nml(nml));
        self
    }

    pub fn ray(&self, pos: Vector<F>, dir: Vector<F>) -> Ray<F> {
        let mut ray = Ray::new(pos, dir);
        ray.lvl = self.lvl + 1;
        if self.flags.contains(RF::Debug) {
            ray = ray.with_debug();
        }
        ray
    }

    pub fn reflected_ray(&mut self) -> Ray<F> {
        let refl = self.dir.reflect(&self.nml());
        let nml = self.nml();
        self.ray(self.pos + nml * F::BIAS4, refl)
    }

    pub fn refracted_ray(&mut self, ior: F) -> Ray<F> {
        let refr = self.dir.refract(&self.nml(), ior);
        let nml = self.nml();
        self.ray(self.pos - nml * F::BIAS4, refr)
    }

    pub fn fresnel(&mut self, ior: F) -> F {
        self.dir.fresnel(&self.nml(), ior)
    }

    pub fn shadow_ray(&mut self, lixel: &Lixel<F>) -> Ray<F> {
        let pos = self.pos + self.nml() * F::BIAS2;
        self.ray(pos, lixel.dir)
    }

    #[must_use]
    pub const fn with_normal(self, nml: Vector<F>) -> Self {
        Self {
            nml: Some(nml),
            ..self
        }
    }

    #[must_use]
    pub const fn with_uv(self, uv: Point<F>) -> Self {
        Self {
            uv: Some(uv),
            ..self
        }
    }

    #[must_use]
    pub const fn with_st(self, st: Point<F>) -> Self {
        Maxel {
            st: Some(st),
            ..self
        }
    }

    pub fn uv(&mut self) -> Point<F> {
        match self.uv {
            None => {
                let uv = self.obj.uv(self);
                *self.uv.insert(uv)
            }
            Some(p) => p,
        }
    }

    pub fn st(&mut self) -> Point<F> {
        match self.st {
            None => {
                let st = self.obj.st(self);
                *self.st.insert(st)
            }
            Some(p) => p,
        }
    }

    pub fn nml(&mut self) -> Vector<F> {
        match self.nml {
            None => {
                let nml = self.obj.normal(self);
                *self.nml.insert(nml)
            }
            Some(p) => p,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    use cgmath::InnerSpace;

    use crate::types::{Vector, Vectorx};

    macro_rules! assert_vec {
        ($val:expr, $x:expr, $y:expr, $z:expr) => {
            assert_f64_near!($val.x, $x);
            assert_f64_near!($val.y, $y);
            assert_f64_near!($val.z, $z);
        };
    }

    #[test]
    fn test_reflect() {
        let dir = Vector::new(1.0, -1.0, 0.0).normalize();
        let nml = Vector::new(0.0, 1.0, 0.0);
        assert_vec!(dir.reflect(&nml), dir.x, -dir.y, 0.0);
    }
}
