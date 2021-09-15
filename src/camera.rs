use crate::traits::Float;
use crate::vector::Vector;
use crate::ray::Ray;
use crate::point::Point;

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
        let dir = (lookat - pos).normalized();

        info!("Camera::raw [ pos:{:?},  dir:{:?},  hor:{:?},  ver:{:?} ]", pos, dir, hor, ver);
        Camera { pos, dir, hor, ver, xres, yres }
    }

    pub fn parametric(
        pos: Vector<F>,
        lookat: Vector<F>,
        fov: F,
        xres: usize,
        yres: usize
    ) -> Camera<F>
    {
        let dir = pos.normal_to(lookat);
        let u = dir.cross(Vector::identity_y()).normalized();
        let v = u.cross(dir).normalized();
        let aspect_ratio = F::from_usize(yres) / F::from_usize(xres);
        let viewplane_width = (fov / F::TWO).tan();
        let viewplane_height = aspect_ratio * viewplane_width;
        let x_inc_vector = (u * viewplane_width)  / F::from_usize(xres);
        let y_inc_vector = (v * viewplane_height) / F::from_usize(yres);
        info!("aspect_ratio: {}", aspect_ratio);
        info!("vp_width: {:.4}", viewplane_width);
        info!("vp_height: {:.4}", viewplane_height);
        info!("x_inc_vector: {:8.4}", x_inc_vector);
        info!("y_inc_vector: {:8.4}", y_inc_vector);

        Camera {
            pos,
            dir: pos.normal_to(lookat),
            hor: x_inc_vector,
            ver: y_inc_vector,
            xres,
            yres
        }
    }

    pub fn get_ray(self, point: Point<F>) -> Ray<F>
    {
        let vpp = self.dir + (self.hor * point.x) + (self.ver * point.y);
        Ray::new(self.pos, vpp.normalized())
    }

    pub fn size(self) -> (usize, usize)
    {
        (self.xres, self.yres)
    }

    pub fn length_to(self, pos: Vector<F>) -> F
    {
        pos.length_to(self.pos)
    }
}
