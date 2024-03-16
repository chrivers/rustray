use super::Float;
use crate::types::ray::{Maxel, Ray};
use crate::geometry::Geometry;

use rtbvh::Primitive;

use cgmath::MetricSpace;

pub trait BvhExt
{
    fn nearest_intersection<'a, F, T>(&'a self, ray: &Ray<F>, prims: &'a [T], dist: &mut F) -> Option<Maxel<'a, F>>
    where
        F: Float,
        T: Primitive + Geometry<F> + 'a;
}

impl BvhExt for rtbvh::Bvh
{
    fn nearest_intersection<'a, F, T>(&'a self, ray: &Ray<F>, prims: &'a [T], dist: &mut F) -> Option<Maxel<'a, F>>
    where
        F: Float,
        T: Primitive + Geometry<F> + 'a
    {
        let mut r: rtbvh::Ray = ray.into();

        let mut hit: Option<Maxel<F>> = None;
        for (t, _) in self.traverse_iter(&mut r, prims) {
            if let Some(curhit) = t.intersect(ray)
            {
                let curdist = ray.pos.distance2(curhit.pos);
                if curdist > F::BIAS2 && curdist < *dist
                {
                    *dist = curdist;
                    hit = Some(curhit);
                }
            }
        }
        hit
    }
}
