#![allow(dead_code)]

use crate::traits::Float;
use crate::vector::Vector;
use crate::ray::Ray;
use point::Point;

#[derive(Clone, Copy)]
pub struct Camera<F: Float>
{
    pub pos: Vector<F>,
    dir: Vector<F>,
    hor: Vector<F>,
    ver: Vector<F>,
}

impl<F: Float> Camera<F>
{
    pub fn raw(
        pos: Vector<F>,
        lookat: Vector<F>,
        hor: Vector<F>,
        ver: Vector<F>,
    ) -> Camera<F>
    {
        let dir = (lookat - pos).normalized();

        info!("Camera::raw [ pos:{:?},  dir:{:?},  hor:{:?},  ver:{:?} ]", pos, dir, hor, ver);
        Camera { pos, dir, hor, ver }
    }

    pub fn parametric(
        pos: Vector<F>,
        lookat: Vector<F>,
        fov: F,
        xres: usize,
        yres: usize
    ) -> Camera<F>
    {
        let dir = (lookat - pos).normalized();
        let u = dir.crossed(Vector::new(F::zero(), F::one(), F::zero()));
        let v = u.crossed(dir);
        let u = u.normalized();
        let v = v.normalized();
        let aspect_ratio = F::from_u32(yres as u32) / F::from_u32(xres as u32);
        let viewplane_half_width = (fov / F::from_u32(2)).tan();
        let viewplane_half_height = aspect_ratio * viewplane_half_width;
        let viewplane_bottom_left = lookat - (v * viewplane_half_height) - (u * viewplane_half_width);
        let x_inc_vector = (u * F::from_u32(2) * viewplane_half_width)  / F::from_u32(xres as u32);
        let y_inc_vector = (v * F::from_u32(2) * viewplane_half_height) / F::from_u32(yres as u32);
        info!("aspect_ratio: {}", aspect_ratio);
        info!("vp_half_width: {}", viewplane_half_width);
        info!("vp_half_height: {}", viewplane_half_height);
        info!("vp_bottom_left: {:?}", viewplane_bottom_left);
        info!("x_int_vector: {:?}", x_inc_vector);
        info!("y_int_vector: {:?}", y_inc_vector);

        Camera::raw(pos, viewplane_bottom_left, x_inc_vector, y_inc_vector)
    }

    pub fn get_ray(self, point: Point<F>) -> Ray<F>
    {
        let one_half = F::from_float(0.5);
        let dir = self.dir +
            self.hor * (point.x - one_half) -
            self.ver * (point.y - one_half);
        Ray::new(self.pos, dir.normalized())
    }
}
