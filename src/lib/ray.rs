use super::{Float, Point, Vector};
use crate::scene::HitTarget;
use crate::material::Material;
use crate::point;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub lvl: u32,
}

pub struct Hit<'a, F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub obj: &'a dyn HitTarget<F>,
    pub lvl: u32,
}

#[derive(Copy, Clone)]
pub struct Maxel<'a, F: Float>
{
    pub normal: Vector<F>,
    pub normalu: Vector<F>,
    pub normalv: Vector<F>,
    pub uv: Point<F>,
    pub mat: &'a dyn Material<F=F>,
}

impl<'a, F: Float> Ray<F>
{
    pub fn new(pos: Vector<F>, dir: Vector<F>, lvl: u32) -> Ray<F>
    {
        Ray { pos, dir, lvl }
    }

    pub fn length_to(self, other: Vector<F>) -> F
    {
        self.dir.cross(self.pos.vector_to(other)).length() / self.dir.length()
    }

    pub fn extend(self, scale: F) -> Vector<F>
    {
        self.pos + self.dir * scale
    }

    pub fn hit_at(self, ext: F, obj: &'a dyn HitTarget<F>) -> Hit<'a, F>
    {
        Hit { pos: self.extend(ext), dir: self.dir, obj, lvl: self.lvl }
    }

    pub fn intersect_sphere(&self, pos: &Vector<F>, radius2: F) -> Option<F>
    {
        let l = self.pos - *pos;
        let a = self.dir.len_sqr();
        let b = F::TWO * l.dot(self.dir);
        let c = l.dot(l) - radius2;

        quadratic(a, b, c)
    }

    pub fn intersect_plane(&self, pos: &Vector<F>, dir1: &Vector<F>, dir2: &Vector<F>) -> Option<F>
    {
        let abc = dir1.cross(*dir2);
        let d = abc.dot(*pos);
        let t = (-abc.dot(self.pos) + d) / abc.dot(self.dir);

        if t < F::epsilon() {
            None
        } else {
            Some(t)
        }
    }

    /**
      Implementation of the Möller–Trumbore intersection algorithm, adapted from
      the reference algorithm on Wikipedia:

      https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm
     */
    pub fn intersect_triangle(&self, a: &Vector<F>, b: &Vector<F>, c: &Vector<F>, n: &Vector<F>) -> Option<F>
    {
        let edge1 = *b - *a;
        let edge2 = *c - *a;

        let h = self.dir.cross(edge2);
        let ae = edge1.dot(h);

        /* This ray is parallel to this triangle. */
        if ae.abs() < F::BIAS {
            return None
        }

        let f = F::one() / ae;

        let s = self.pos - *a;
        let u = f * s.dot(h);
        if u < F::zero() || u > F::one() {
            return None
        }

        let q = s.cross(edge1);
        let v = f * self.dir.dot(q);
        if v < F::zero() || u + v > F::one() {
            return None
        }

        /* Compute t to find out where the intersection point is on the line. */
        let t = f * edge2.dot(q);
        if t < F::BIAS {
            /* Line intersection but not a ray intersection. */
            return None
        }

        /* ray intersection */
        Some(t)
    }

}

/* Hit */
impl<'a, F: Float> Hit<'a, F>
{
    pub fn reflected_ray(&self, normal: &Vector<F>) -> Ray<F>
    {
        let refl = self.dir.reflect(&normal);
        Ray::new(self.pos + refl * F::BIAS, refl, self.lvl + 1)
    }

    pub fn refracted_ray(&self, normal: &Vector<F>, ior: F) -> Ray<F>
    {
        let refr = self.dir.refract(&normal, ior);
        Ray::new(self.pos + refr * F::BIAS, refr, self.lvl + 1)
    }
}

/* Maxel */

impl<'a, F: Float> Maxel<'a, F>
{
    pub fn from_uv(u: F, v: F, normal: Vector<F>, normalu: Vector<F>, normalv: Vector<F>, mat: &'a dyn Material<F=F>) -> Self
    {
        Maxel { uv: point!(u, v), normal, normalu, normalv, mat }
    }

    pub fn new(uv: Point<F>, normal: Vector<F>, normalu: Vector<F>, normalv: Vector<F>, mat: &'a dyn Material<F=F>) -> Self
    {
        Maxel { uv, normal, normalu, normalv, mat }
    }

    pub fn zero(mat: &'a dyn Material<F=F>) -> Self
    {
        Maxel { uv: Point::zero(), normal: Vector::zero(), normalu: Vector::zero(), normalv: Vector::zero(), mat }
    }
}

/* Math functions */

fn quadratic<F: Float>(a: F, b: F, c: F) -> Option<F>
{
    let discr = b * b - F::FOUR * a * c;

    if discr < F::zero() {
        return None
    }

    let t = {
        let q = if b > F::zero() {
            -F::HALF * (b + discr.sqrt())
        } else {
            -F::HALF * (b - discr.sqrt())
        };
        let t0 = q / a;
        let t1 = c / q;
        t0.min(t1)
    };

    if t >= F::zero() {
        Some(t)
    } else {
        None
    }
}
