#![allow(dead_code)]

use crate::traits::Float;
use crate::vector::Vector;
use crate::ray::Ray;

#[derive(Clone, Copy)]
pub struct Camera<F: Float>
{
    pub pos: Vector<F>,
    dir: Vector<F>,
    hor: Vector<F>,
    ver: Vector<F>,
    x: u32,
    y: u32,
}

impl<F: Float> Camera<F>
{
    pub fn new(
        pos: Vector<F>,
        dir: Vector<F>,
        hor: Vector<F>,
        ver: Vector<F>,
        x: u32,
        y: u32) -> Camera<F>
    {
        Camera { pos: pos, dir: dir, hor: hor, ver: ver, x: x, y: y }
    }

    pub fn get_ray(self, xp: F, yp: F) -> Ray<F>
    {
        let one_half = F::one() / (F::one() + F::one());
        let dir = self.dir +
            self.hor * ( xp - one_half) +
            self.ver * (-yp + one_half);
        Ray::new(self.pos, dir.normalized())
    }
}
