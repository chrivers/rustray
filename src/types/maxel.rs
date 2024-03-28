use core::fmt::Debug;

use super::vector::Vectorx;
use super::{Float, Point, Vector};

use crate::geometry::Geometry;
use crate::material::Material;
use crate::types::ray::Ray;

#[derive(Copy, Clone)]
pub struct Maxel<'a, F: Float> {
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub obj: &'a dyn Geometry<F>,
    pub mat: &'a dyn Material<F>,
    pub lvl: u32,
    nml: Option<Vector<F>>,
    uv: Option<Point<F>>,
    st: Option<Point<F>>,
}

impl<'a, F: Float> Debug for Maxel<'a, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        pos: Vector<F>,
        dir: Vector<F>,
        lvl: u32,
        obj: &'a dyn Geometry<F>,
        mat: &'a dyn Material<F>,
    ) -> Self {
        Maxel {
            pos,
            dir,
            lvl,
            obj,
            mat,
            nml: None,
            uv: None,
            st: None,
        }
    }

    pub fn reflected_ray(&mut self) -> Ray<F> {
        let refl = self.dir.reflect(&self.nml());
        Ray::new(self.pos + refl * F::BIAS4, refl, self.lvl + 1)
    }

    pub fn refracted_ray(&mut self, ior: F) -> Ray<F> {
        let refr = self.dir.refract(&self.nml(), ior);
        Ray::new(self.pos + refr * F::BIAS4, refr, self.lvl + 1)
    }

    pub fn fresnel(&mut self, ior: F) -> F {
        self.dir.fresnel(&self.nml(), ior)
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
