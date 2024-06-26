use cgmath::InnerSpace;
use flagset::{flags, FlagSet};

use crate::geometry::Geometry;
use crate::types::{Float, MaterialId, Maxel, Transform, Vector, Vectorx};

flags! {
    pub enum RF: u16 {
        Debug,
        StopAtGroup,
        Preview,
    }
}

pub type RayFlags = FlagSet<RF>;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float> {
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub lvl: u16,
    pub flags: RayFlags,
}

impl<'a, F: Float> Ray<F> {
    pub const fn new(pos: Vector<F>, dir: Vector<F>) -> Self {
        Self {
            pos,
            dir,
            lvl: 0,
            flags: RayFlags::default(),
        }
    }

    #[must_use]
    pub const fn with_debug(self) -> Self {
        self.with_flags(RF::Debug.into())
    }

    #[must_use]
    #[allow(clippy::assign_op_pattern)]
    pub const fn with_flags(mut self, flags: RayFlags) -> Self {
        self.flags = self.flags | flags;
        self
    }

    #[must_use]
    pub fn extend(self, scale: F) -> Vector<F> {
        self.pos + self.dir * scale
    }

    pub fn hit_at(
        self,
        hit: Vector<F>,
        ext: F,
        obj: &'a dyn Geometry<F>,
        mat: MaterialId,
    ) -> Maxel<'a, F> {
        Maxel::new(
            hit,
            self.extend(ext),
            self.dir,
            self.lvl,
            obj,
            mat,
            self.flags,
        )
    }

    pub fn synthetic_hit<G: Geometry<F>>(self, center: Vector<F>, obj: &'a G) -> Maxel<'a, F> {
        Maxel::new(
            Vector::ZERO,
            center,
            -self.dir,
            self.lvl,
            obj,
            MaterialId::NULL,
            self.flags,
        )
    }

    pub fn enter_group(self) -> Option<Self> {
        if self.flags.contains(RF::StopAtGroup) {
            None
        } else {
            Some(self)
        }
    }

    #[must_use]
    pub fn xfrm_inv(&self, xfrm: &Transform<F>) -> Self {
        Self {
            pos: xfrm.pos_inv(self.pos),
            dir: xfrm.dir_inv(self.dir),
            ..*self
        }
    }

    #[must_use]
    pub fn xfrm(&self, xfrm: &Transform<F>) -> Self {
        Self {
            pos: xfrm.pos(self.pos),
            dir: xfrm.dir(self.dir),
            ..*self
        }
    }

    pub fn intersect_sphere(&self, pos: &Vector<F>, radius2: F) -> Option<F> {
        let l = self.pos - *pos;
        let a = self.dir.magnitude2();
        let b = F::TWO * l.dot(self.dir);
        let c = l.dot(l) - radius2;

        super::quadratic(a, b, c)
    }

    pub fn intersect_unit_sphere(&self) -> Option<F> {
        let a = self.dir.dot(self.dir);
        let b = F::TWO * self.dir.dot(self.pos);
        let c = self.pos.dot(self.pos) - F::ONE;

        super::quadratic(a, b, c)
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
        if !u.is_unit() {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * self.dir.dot(q);
        if v.is_negative() || u + v > F::ONE {
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

        if w.is_negative() {
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
        Self::new(ray.pos.into_vec3(), ray.dir.into_vec3())
    }
}

impl<F: Float> From<&Ray<F>> for rtbvh::Ray {
    fn from(ray: &Ray<F>) -> Self {
        From::from(*ray)
    }
}
