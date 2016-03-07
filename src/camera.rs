#![allow(dead_code)]

use vector::Vector;
use ray::Ray;
use num::Float;

#[derive(Clone, Copy)]
pub struct Camera<F: Float>
{
    pos: Vector<F>,
    dir: Vector<F>,
    hor: Vector<F>,
    ver: Vector<F>,
    x: u32,
    y: u32,
}

impl<F: Float> Camera<F>
{
    pub fn get_ray(self, xp: F, yp: F) -> Ray<F>
    {
        let one_half = F::one() / (F::one() + F::one());
        let dir = self.dir +
            self.hor * ( xp - one_half) +
            self.ver * (-yp + one_half);
        Ray::new(self.pos, dir.normalized())
    }
}
