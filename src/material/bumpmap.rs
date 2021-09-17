use super::mat_util::*;

use image::GenericImageView;
use image::{Pixel, Rgb};

use num_traits::ToPrimitive;

#[derive(Copy, Clone)]
pub struct Bumpmap<F: Float, I: GenericImageView + Sync, M: Material<F=F>>
{
    pow: F,
    img: I,
    mat: M
}

impl<F: Float, I: GenericImageView + Sync, M: Material<F=F>> Bumpmap<F, I, M>
{
    pub fn new(pow: F, img: I, mat: M) -> Self
    {
        Self { pow, img, mat }
    }
}

impl<F: Float, I: GenericImageView + Sync, M: Material<F=F>> Material for Bumpmap<F, I, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let x: u32 = (maxel.uv.x.abs() * F::from_u32(256)).to_u32().unwrap_or(0) % 254;
        let y: u32 = (maxel.uv.y.abs() * F::from_u32(256)).to_u32().unwrap_or(0) % 254;
        let fx = (maxel.uv.x.abs() * F::from_u32(256)).fract();
        let fy = (maxel.uv.y.abs() * F::from_u32(256)).fract();
        let nfx = F::one() - fx;
        let nfy = F::one() - fy;
        let Rgb([r, g, b]) = self.img.get_pixel(x, y).to_rgb();
        let n1 = vec3!(
            F::from_f32(r.to_f32().unwrap() / 255.0),
            F::from_f32(g.to_f32().unwrap() / 255.0),
            F::from_f32(b.to_f32().unwrap() / 255.0)
        );
        let Rgb([r, g, b]) = self.img.get_pixel(x+1, y).to_rgb();
        let n2 = vec3!(
            F::from_f32(r.to_f32().unwrap() / 255.0),
            F::from_f32(g.to_f32().unwrap() / 255.0),
            F::from_f32(b.to_f32().unwrap() / 255.0)
        );
        let Rgb([r, g, b]) = self.img.get_pixel(x, y+1).to_rgb();
        let n3 = vec3!(
            F::from_f32(r.to_f32().unwrap() / 255.0),
            F::from_f32(g.to_f32().unwrap() / 255.0),
            F::from_f32(b.to_f32().unwrap() / 255.0)
        );
        let Rgb([r, g, b]) = self.img.get_pixel(x+1, y+1).to_rgb();
        let n4 = vec3!(
            F::from_f32(r.to_f32().unwrap() / 255.0),
            F::from_f32(g.to_f32().unwrap() / 255.0),
            F::from_f32(b.to_f32().unwrap() / 255.0)
        );
        let n =
            ((n1 * nfx) + n2 * fx) * nfy +
            ((n3 * nfx) + n4 * fx) * fy;
        // Color::new(n.x, n.y, n.z)
        let mut mxl = *maxel;
        mxl.normal = (n * self.pow).normalized();
        // mxl.normal = (maxel.normal + n * self.pow).normalized();
        // info!("{:?}", n.x);
        // mxl.uv.x += (n.x - F::HALF) * self.pow;
        // mxl.uv.y += (n.y - F::HALF) * self.pow;
        self.mat.render(hit, &mxl, lights, rt, lvl)
        // Color::new(n.x, n.y, n.z)
        // let refl = (maxel.normal + n * self.pow).normalized();
        // let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);
        // c_refl
    }
}
