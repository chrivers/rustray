use super::vector::{InnerSpace, Vectorx};
use super::{Float, Vector};
use crate::geometry::Geometry;
use crate::material::Material;
use crate::types::transform::Transform;

use cgmath::{EuclideanSpace, Matrix4, Point3, Transform as cgTransform};

pub use super::maxel::Maxel;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float> {
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub lvl: u32,
}

impl<'a, F: Float> Ray<F> {
    pub fn new(pos: Vector<F>, dir: Vector<F>, lvl: u32) -> Ray<F> {
        Ray { pos, dir, lvl }
    }

    pub fn extend(self, scale: F) -> Vector<F> {
        self.pos + self.dir * scale
    }

    pub fn hit_at(
        self,
        ext: F,
        obj: &'a dyn Geometry<F>,
        mat: &'a dyn Material<F = F>,
    ) -> Maxel<'a, F> {
        Maxel::new(self.extend(ext), self.dir, self.lvl, obj, mat)
    }

    pub fn inverse_transform(&self, xfrm: &Matrix4<F>) -> Option<Ray<F>> {
        let inv = xfrm.inverse_transform()?;
        self.transform(&inv)
    }

    pub fn transform(&self, xfrm: &Matrix4<F>) -> Option<Ray<F>> {
        Some(Self {
            pos: xfrm.transform_point(Point3::from_vec(self.pos)).to_vec(),
            dir: xfrm.transform_vector(self.dir),
            lvl: self.lvl,
        })
    }

    pub fn xfrm_inv(&self, xfrm: &Transform<F>) -> Ray<F> {
        Self {
            pos: xfrm.pos_inv(self.pos),
            dir: xfrm.dir_inv(self.dir),
            lvl: self.lvl,
        }
    }

    pub fn xfrm(&self, xfrm: &Transform<F>) -> Ray<F> {
        Self {
            pos: xfrm.pos(self.pos),
            dir: xfrm.dir(self.dir),
            lvl: self.lvl,
        }
    }

    pub fn intersect_sphere(&self, pos: &Vector<F>, radius2: F) -> Option<F> {
        let l = self.pos - *pos;
        let a = self.dir.magnitude2();
        let b = F::TWO * l.dot(self.dir);
        let c = l.dot(l) - radius2;

        quadratic(a, b, c)
    }

    pub fn intersect_plane(
        &self,
        pos: &Vector<F>,
        dir1: &Vector<F>,
        dir2: &Vector<F>,
    ) -> Option<F> {
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

    > [https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm](https://en.wikipedia.org/wiki/M%c3%b6ller%e2%80%93Trumbore_intersection_algorithm)
     */
    pub fn intersect_triangle(&self, a: &Vector<F>, b: &Vector<F>, c: &Vector<F>) -> Option<F> {
        let edge1 = *b - *a;
        let edge2 = *c - *a;

        let h = self.dir.cross(edge2);
        let ae = edge1.dot(h);

        /* This ray is parallel to this triangle. */
        if ae.abs() < F::BIAS2 {
            return None;
        }

        let f = F::ONE / ae;

        let s = self.pos - *a;
        let u = f * s.dot(h);
        if u < F::ZERO || u > F::ONE {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * self.dir.dot(q);
        if v < F::ZERO || u + v > F::ONE {
            return None;
        }

        /* Compute t to find out where the intersection point is on the line. */
        let t = f * edge2.dot(q);
        if t < F::BIAS {
            /* Line intersection but not a ray intersection. */
            return None;
        }

        /* ray intersection */
        Some(t)
    }

    /**
    Implementation of the "new algorithm" for segment/triangle intersection,
    adapted from the paper:

    > "A robust segment/triangle intersection algorithm for interference tests. Efficiency study" \
    > [*Jiménez, Segura, Feito.*]

    (this version considers only front faces)
     */
    pub fn intersect_triangle2(&self, v1: &Vector<F>, v2: &Vector<F>, v3: &Vector<F>) -> Option<F> {
        let scale = F::from_f32(1e7);
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
            if s > F::BIAS {
                return None;
            }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t < -F::BIAS {
                return None;
            }
            u = -w2.dot(b);
            if u < -F::BIAS {
                return None;
            }
            if w < s + t + u {
                return None;
            }
        } else if w < -F::BIAS {
            return None;
        } else {
            let d = q2 - v3;
            s = d.dot(w1);
            if s > F::BIAS {
                return None;
            } else if s < -F::BIAS {
                let w2 = d.cross(a);
                t = w2.dot(c);
                if t > F::BIAS {
                    return None;
                }
                u = -w2.dot(b);
                if u > F::BIAS {
                    return None;
                }
                if -s > t + u {
                    return None;
                }
            } else {
                return None;
            }
        }

        // let alpha = tt * t;
        // let beta = tt * u;
        Some((scale * w) / (w - s))
    }

    /**
    Implementation of the "new algorithm" for segment/triangle intersection,
    adapted from the paper:

    > "A robust segment/triangle intersection algorithm for interference tests. Efficiency study" \
    > [*Jiménez, Segura, Feito.*]

    (this version considers both front and back faces)
     */
    pub fn intersect_triangle3(&self, v1: &Vector<F>, v2: &Vector<F>, v3: &Vector<F>) -> Option<F> {
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
            if s > F::BIAS {
                return None;
            }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t < -F::BIAS {
                return None;
            }
            u = -w2.dot(b);
            if u < -F::BIAS {
                return None;
            }
            if w < s + t + u {
                return None;
            }
        } else if w < -F::BIAS {
            if s < -F::BIAS {
                return None;
            }
            let w2 = a.cross(d);
            t = w2.dot(c);
            if t > F::BIAS {
                return None;
            }
            u = -w2.dot(b);
            if u > F::BIAS {
                return None;
            }
            if w > s + t + u {
                return None;
            }
        } else if s > F::BIAS {
            let w2 = d.cross(a);
            t = w2.dot(c);
            if t < -F::BIAS {
                return None;
            }
            u = -w2.dot(b);
            if u < -F::BIAS {
                return None;
            }
            if -s < t + u {
                return None;
            }
        } else if s < -F::BIAS {
            let w2 = d.cross(a);
            t = w2.dot(c);
            if t > F::BIAS {
                return None;
            }
            u = -w2.dot(b);
            if u > F::BIAS {
                return None;
            }
            if -s > t + u {
                return None;
            }
        } else {
            return None;
        }

        Some((scale * w) / (w - s))
    }

    /**
    Implementation of the "new algorithm" for segment/triangle intersection,
    adapted from the paper:

    > "A robust segment/triangle intersection algorithm for interference tests. Efficiency study" \
    > [*Jiménez, Segura, Feito.*]

    (this version considers only front faces)
     */
    pub fn intersect_triangle4(&self, e1: &Vector<F>, e2: &Vector<F>, v3: &Vector<F>) -> Option<F> {
        let scale = F::from_f32(1e4);
        let a = self.pos - v3;
        let w1 = e1.cross(*e2);
        let w = a.dot(w1);

        if w < F::ZERO {
            return None;
        }

        let d = a + self.dir * scale;

        let s = d.dot(w1);
        if s > -F::BIAS {
            return None;
        }

        let w2 = a.cross(d);

        let t = w2.dot(*e2);
        if t < -F::BIAS {
            return None;
        }

        let u = -w2.dot(*e1);
        if u < -F::BIAS {
            return None;
        }

        if w < s + t + u {
            return None;
        }
        Some(w / (w - s) * scale)
    }
}

impl<F: Float> From<Ray<F>> for rtbvh::Ray {
    fn from(ray: Ray<F>) -> Self {
        Self::new(ray.pos.into_vector3(), ray.dir.into_vector3())
    }
}

impl<F: Float> From<&Ray<F>> for rtbvh::Ray {
    fn from(ray: &Ray<F>) -> Self {
        From::from(*ray)
    }
}

/* Math functions */

pub fn quadratic<F: Float>(a: F, b: F, c: F) -> Option<F> {
    let (t0, t1) = quadratic2(a, b, c)?;
    if t0 < F::ZERO || t1 < F::ZERO {
        None
    } else {
        Some(t0.min(t1))
    }
}

pub fn quadratic2<F: Float>(a: F, b: F, c: F) -> Option<(F, F)> {
    let discr = b * b - F::FOUR * a * c;

    if discr < F::ZERO {
        return None;
    }

    let q = if b > F::ZERO {
        -F::HALF * (b + discr.sqrt())
    } else {
        -F::HALF * (b - discr.sqrt())
    };
    let t0 = q / a;
    let t1 = c / q;
    if t0 < t1 {
        Some((t0, t1))
    } else {
        Some((t1, t0))
    }
}
