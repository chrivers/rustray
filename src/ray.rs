use crate::traits::Float;
use crate::vector::Vector;
use crate::point::Point;
use crate::scene::RayTarget;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
}

pub struct Hit<'a, F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub uv: Point<F>,
    pub obj: &'a dyn RayTarget<F>,
}

impl<F: Float> Ray<F>
{
    pub fn new(pos: Vector<F>, dir: Vector<F>) -> Ray<F>
    {
        Ray { pos, dir }
    }

    pub fn length_to(self, other: Vector<F>) -> F
    {
        self.dir.cross(self.pos.vector_to(other)).length() / self.dir.length()
    }

    pub fn extend(self, scale: F) -> Vector<F>
    {
        self.pos + self.dir * scale
    }

    pub fn hit_at(self, ext: F, uv: Point<F>, obj: &dyn RayTarget<F>) -> Hit<'_, F>
    {
        Hit { pos: self.extend(ext), dir: self.dir, uv, obj }
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

    pub fn intersect_triangle(&self, a: &Vector<F>, b: &Vector<F>, c: &Vector<F>, n: &Vector<F>) -> Option<(F, F, F)>
    {
        fn test_edge<F: Float>(edge: Vector<F>, vp: Vector<F>, normal: &Vector<F>) -> Option<Vector<F>>
        {
            let c = edge.cross(vp);
            if normal.dot(c) < F::zero() {
                None
            } else {
                Some(c)
            }
        }

        let t = self.intersect_plane(&a, &(*b - *a), &(*c - *a))?;
        let hit = self.extend(t);

        let c0 = test_edge(*b - *a, hit - *a, &n)?;
        let c1 = test_edge(*c - *b, hit - *b, &n)?;
        let c2 = test_edge(*a - *c, hit - *c, &n)?;

        let area2 = n.length();
        let u = c1.length() / area2;
        let v = c2.length() / area2;

        Some((t, u, v))
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
