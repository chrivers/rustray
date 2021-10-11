use super::Float;
use super::Vector;
use super::Ray;
use super::Point;
use super::vector::{InnerSpace, MetricSpace};
use cgmath::{Deg, Angle};

#[derive(Clone, Copy, Debug)]
pub struct Camera<F: Float>
{
    pos: Vector<F>,
    dir: Vector<F>,
    hor: Vector<F>,
    ver: Vector<F>,
    xres: usize,
    yres: usize,
}

impl<F: Float> Camera<F>
{
    pub fn raw(
        pos: Vector<F>,
        lookat: Vector<F>,
        hor: Vector<F>,
        ver: Vector<F>,
        xres: usize,
        yres: usize,
    ) -> Camera<F>
    {
        let dir = (lookat - pos).normalize();

        info!("Camera::raw [ pos:{:?},  dir:{:?},  hor:{:?},  ver:{:?} ]", pos, dir, hor, ver);
        Camera { pos, dir, hor, ver, xres, yres }
    }

    pub fn parametric(
        pos: Vector<F>,
        lookat: Vector<F>,
        updir: Vector<F>,
        fov: F,
        xres: usize,
        yres: usize
    ) -> Camera<F>
    {
        Self::build(pos, lookat - pos, updir, fov, xres, yres, None)
    }

    pub fn build(
        pos: Vector<F>,
        viewdir: Vector<F>,
        updir: Vector<F>,
        fov: F,
        xres: usize,
        yres: usize,
        aspect: Option<F>,
    ) -> Camera<F>
    {
        let dir = viewdir.normalize();
        let u = dir.cross(updir).normalize();
        let v = u.cross(dir).normalize();
        let aspect_ratio = aspect.unwrap_or_else(|| F::from_usize(xres) / F::from_usize(yres));
        let viewplane_height = Deg(fov / F::TWO).tan() * F::TWO;
        let viewplane_width = aspect_ratio * viewplane_height;
        let x_inc_vector = u * viewplane_width;
        let y_inc_vector = v * viewplane_height;
        info!("aspect_ratio: {}", aspect_ratio);
        info!("vp_width: {:.4}", viewplane_width);
        info!("vp_height: {:.4}", viewplane_height);
        info!("x_inc_vector: {:8.4?}", x_inc_vector);
        info!("y_inc_vector: {:8.4?}", y_inc_vector);

        Camera {
            pos,
            dir,
            hor: x_inc_vector,
            ver: y_inc_vector,
            xres,
            yres,
        }
    }

    pub fn get_ray(self, point: Point<F>) -> Ray<F>
    {
        let x = point.x - F::HALF;
        let y = -point.y + F::HALF;
        let vpp = self.dir + (self.hor * x) + (self.ver * y);
        Ray::new(self.pos, vpp.normalize(), 0)
    }

    pub fn size(self) -> (usize, usize)
    {
        (self.xres, self.yres)
    }

    pub fn distance2(self, pos: Vector<F>) -> F
    {
        self.pos.distance2(pos)
    }
}
