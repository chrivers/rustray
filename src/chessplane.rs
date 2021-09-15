use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::{Ray, Hit};
use crate::point::Point;

#[derive(Debug)]
pub struct ChessPlane<F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
}

impl<F: Float> RayTarget<F> for ChessPlane<F>
{
    fn resolve(&self, hit: &Hit<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, _lvl: u32) -> Color<F>
    {
        let xs = F::from_f32(2.0);
        let ys = F::from_f32(2.0);

        let s;
        let t;

        if self.dir1.x.non_zero() {
            s = hit.pos.x / self.dir1.x;
            if self.dir2.y.non_zero() {
                t = hit.pos.y / self.dir2.y;
            } else {
                t = hit.pos.z / self.dir2.z;
            }
        } else {
            s = hit.pos.y / self.dir1.y;
            if self.dir2.x.non_zero() {
                t = hit.pos.x / self.dir2.x;
            } else {
                t = hit.pos.z / self.dir2.z;
            }
        }
        let xv = s / xs;
        let yv = t / ys;

        let x = xv.abs().fract() > F::HALF;
        let y = yv.abs().fract() > F::HALF;

        let self_color = if x^y {
            Color::black()
        } else {
            Color::white()
        };

        let mut res = Color::black();
        for light in lights {
            if rt.ray_shadow(hit, light) {
                continue
            }
            let m = hit.pos.vector_to(light.pos);
            let light_color = light.color * self_color;
            let reflection_coeff = self.normal.cos_angle(m);
            res += light_color * reflection_coeff / m.length();
        }
        res
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, Point::zero(), self))
    }

}

impl<F: Float> ChessPlane<F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, _color: Color<F>) -> ChessPlane<F>
    {
        ChessPlane { pos, dir1, dir2, normal: dir1.cross(dir2) }
    }
}
