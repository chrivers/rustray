use super::{Float, Point, Vector};
use crate::scene::HitTarget;
use crate::material::Material;
use crate::point;
use super::vector::{Vectorx, InnerSpace};
use num_traits::Zero;

use cgmath::{Point3, Matrix4, Transform, EuclideanSpace};

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
    pub nml: Option<Vector<F>>,
    pub uv:  Option<Point<F>>,
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

    pub fn extend(self, scale: F) -> Vector<F>
    {
        self.pos + self.dir * scale
    }

    pub fn hit_at(self, ext: F, obj: &'a dyn HitTarget<F>) -> Hit<'a, F>
    {
        Hit { pos: self.extend(ext), dir: self.dir, obj, lvl: self.lvl, nml: None, uv: None }
    }

    pub fn inverse_transform(&self, xfrm: &Matrix4<F>) -> Option<Ray<F>>
    {
        let inv = xfrm.inverse_transform()?;
        Some(Self {
            pos: inv.transform_point(Point3::from_vec(self.pos)).to_vec(),
            dir: inv.transform_vector(self.dir),
            lvl: self.lvl,
        })
    }

    pub fn intersect_sphere(&self, pos: &Vector<F>, radius2: F) -> Option<F>
    {
        let l = self.pos - *pos;
        let a = self.dir.magnitude2();
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
    pub fn intersect_triangle(&self, a: &Vector<F>, b: &Vector<F>, c: &Vector<F>) -> Option<F>
    {
        let edge1 = *b - *a;
        let edge2 = *c - *a;

        let h = self.dir.cross(edge2);
        let ae = edge1.dot(h);

        /* This ray is parallel to this triangle. */
        if ae.abs() < F::BIAS {
            return None
        }

        let f = F::ONE / ae;

        let s = self.pos - *a;
        let u = f * s.dot(h);
        if u < F::ZERO || u > F::ONE {
            return None
        }

        let q = s.cross(edge1);
        let v = f * self.dir.dot(q);
        if v < F::ZERO || u + v > F::ONE {
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

    /*
    Implementation of the "new algorithm" for segment/triangle intersection,
    adapted from the paper:

      "A robust segment/triangle intersection algorithm for interference
      tests. Efficiency study" - by Jiménez, Segura, Feito.

    (this version considers only front faces)
     */
    pub fn intersect_triangle2(&self, v1: &Vector<F>, v2: &Vector<F>, v3: &Vector<F>) -> Option<F>
    {
        let scale = F::from_f32(1024.0);
        let q1 = self.pos;
        let q2 = self.pos + self.dir * scale;
        let a = q1 - v3;
        let b = v1 - v3;
        let c = v2 - v3;
        let w1 = b.cross(c);
        let w = a.dot(w1);

        let s;
        let t;
        let u;

        if w > F::BIAS {
            let d = q2 - v3;
            s = d.dot(w1);
            if s > F::BIAS { return None }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t < -F::BIAS { return None }
            u = -w2.dot(b);
            if u < -F::BIAS { return None }
            if w < s + t + u { return None }
        } else if w < -F::BIAS {
            return None
        } else {
            let d = q2 - v3;
            s = d.dot(w1);
            if s > F::BIAS {
                return None
            } else if s < -F::BIAS {
                let w2 = d.cross(a);
                t = w2.dot(c);
                if t > F::BIAS { return None }
                u = -w2.dot(b);
                if u > F::BIAS { return None }
                if -s > t + u { return None }
            } else {
                return None
            }
        }

        // let alpha = tt * t;
        // let beta = tt * u;
        Some((scale * w) / (w - s))
    }

    /*
    Implementation of the "new algorithm" for segment/triangle intersection,
    adapted from the paper:

      "A robust segment/triangle intersection algorithm for interference
      tests. Efficiency study" - by Jiménez, Segura, Feito.

    (this version considers both front and back faces)
     */
    pub fn intersect_triangle3(&self, v1: &Vector<F>, v2: &Vector<F>, v3: &Vector<F>) -> Option<F>
    {
        let scale = F::from_f32(1024.0);
        let q1 = self.pos;
        let q2 = self.pos + self.dir * scale;
        let a = q1 - v3;
        let b = v1 - v3;
        let c = v2 - v3;
        let w1 = b.cross(c);
        let w = a.dot(w1);
        let d = q2 - v3;
        let s = d.dot(w1);

        let t;
        let u;

        if w > F::BIAS {
            if s > F::BIAS { return None }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t < -F::BIAS { return None }
            u = -w2.dot(b);
            if u < -F::BIAS { return None }
            if w < s + t + u { return None }
        } else if w < -F::BIAS {
            if s < -F::BIAS { return None }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t > F::BIAS { return None }
            u = -w2.dot(b);
            if u > F::BIAS { return None }
            if w > s + t + u { return None }
        } else if s > F::BIAS {
            let w2 = d.cross(a);
            t = w2.dot(c);
            if t < -F::BIAS { return None }
            u = -w2.dot(b);
            if u < -F::BIAS { return None }
            if -s < t + u { return None }
        } else if s < -F::BIAS {
            let w2 = d.cross(a);
            t = w2.dot(c);
            if t > F::BIAS { return None }
            u = -w2.dot(b);
            if u > F::BIAS { return None }
            if -s > t + u { return None }
        } else {
            return None
        }

        Some((scale * w) / (w - s))
    }

}

impl<F: Float> Into<bvh::ray::Ray> for Ray<F>
{
    fn into(self) -> bvh::ray::Ray
    {
        bvh::ray::Ray::new(
            self.pos.into_vector3(),
            self.dir.into_vector3(),
        )
    }
}

impl<F: Float> Into<bvh::ray::Ray> for &Ray<F>
{
    fn into(self) -> bvh::ray::Ray
    {
        (*self).into()
    }
}

/* Hit */
impl<'a, F: Float> Hit<'a, F>
{
    pub fn reflected_ray(&self, normal: &Vector<F>) -> Ray<F>
    {
        let refl = self.dir.reflect(normal);
        Ray::new(self.pos + refl * F::BIAS3, refl, self.lvl + 1)
    }

    pub fn refracted_ray(&self, normal: &Vector<F>, ior: F) -> Ray<F>
    {
        let refr = self.dir.refract(normal, ior);
        Ray::new(self.pos + refr * F::BIAS3, refr, self.lvl + 1)
    }

    pub fn with_normal(self, nml: Vector<F>) -> Self
    {
        Self { nml: Some(nml), ..self }
    }

    pub fn with_uv(self, uv: Point<F>) -> Self
    {
        Self { uv: Some(uv), ..self }
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

    if discr < F::ZERO {
        return None
    }

    let t = {
        let q = if b > F::ZERO {
            -F::HALF * (b + discr.sqrt())
        } else {
            -F::HALF * (b - discr.sqrt())
        };
        let t0 = q / a;
        let t1 = c / q;
        t0.min(t1)
    };

    if t >= F::ZERO {
        Some(t)
    } else {
        None
    }
}
