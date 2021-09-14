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
        let dir = (lookat - pos).normalized();
        let u = dir.cross(Vector::new(F::zero(), F::one(), F::zero()));
        let v = u.cross(dir);
        let u = u.normalized();
        let v = v.normalized();
        let aspect_ratio = F::from_u32(yres as u32) / F::from_u32(xres as u32);
        let viewplane_half_width = (fov / F::from_u32(2)).tan();
        let viewplane_half_height = aspect_ratio * viewplane_half_width;
        let viewplane_bottom_left = lookat - (v * viewplane_half_height) - (u * viewplane_half_width);
        let x_inc_vector = (u * F::from_u32(2) * viewplane_half_width)  / F::from_u32(xres as u32);
        let y_inc_vector = (v * F::from_u32(2) * viewplane_half_height) / F::from_u32(yres as u32);
        info!("aspect_ratio: {}", aspect_ratio);
        info!("vp_half_width: {:.4}", viewplane_half_width);
        info!("vp_half_height: {:.4}", viewplane_half_height);
        info!("vp_bottom_left: {:?}", viewplane_bottom_left);
        info!("x_int_vector: {:?}", x_inc_vector);
        info!("y_int_vector: {:?}", y_inc_vector);

        Camera {
            pos,
            dir: (viewplane_bottom_left - pos).normalized(),
            hor: x_inc_vector,
            ver: y_inc_vector,
            xres,
            yres
        }
    }

    pub fn get_ray(self, point: Point<F>) -> Ray<F>
    {
        let vpp = self.dir + (self.hor * (point.x - F::from_u32(0))) + (self.ver * (point.y - F::from_u32(0)));
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
