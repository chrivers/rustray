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
        let (w, h) = self.img.dimensions();
        let x: u32 = (maxel.uv.x * F::from_u32(w)).to_u32().unwrap() % (w-1);
        let y: u32 = (maxel.uv.y * F::from_u32(h)).to_u32().unwrap() % (h-1);
        let fx = (maxel.uv.x.abs() * F::from_u32(w)).fract();
        let fy = (maxel.uv.y.abs() * F::from_u32(h)).fract();
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
        let mut n =
            ((n1 * nfx) + n2 * fx) * nfy +
            ((n3 * nfx) + n4 * fx) * fy;
        n.x -= F::HALF;
        n.y -= F::HALF;
        n.x *= F::TWO;
        n.y *= F::TWO;

        let mut mxl = *maxel;

        let nx =
            mxl.normalu * n.x +
            mxl.normalv * n.y +
            mxl.normal  * n.z / (self.pow + F::BIAS);

        mxl.normal = nx.normalized();

        self.mat.render(hit, &mxl, lights, rt, lvl)
    }
}
