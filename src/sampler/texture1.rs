use image::{GenericImageView, DynamicImage, Pixel, Rgb};
use num_traits::ToPrimitive;

use super::samp_util::*;

#[derive(Copy, Clone)]
/**
`Texture1` implements [`Sampler<F, F>`] by looking up values in a texture
*/
pub struct Texture1<I: GenericImageView + Sync>
{
    /** Backing image */
    img: I,
}

impl<I: GenericImageView + Sync> Texture1<I>
{
    pub fn new(img: I) -> Self
    {
        Self { img }
    }
}

impl<F: Float, I: GenericImageView + Sync> Sampler<F, F> for Texture1<I>
{
    fn sample(&self, uv: Point<F>) -> F
    {
        let (w, h) = self.img.dimensions();
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % w;
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % h;
        let r = self.img.get_pixel(x, y).channels()[0];
        F::from_f32(r.to_f32().unwrap() / 255.0)
    }

    fn raw_sample(&self, uv: Point<u32>) -> F
    {
        let (w, h) = self.img.dimensions();
        let r = self.img.get_pixel(uv.x % w, uv.y % h).channels()[0];
        F::from_f32(r.to_f32().unwrap() / 255.0)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.img.dimensions()
    }
}

impl<F: Float> Sampler<F, F> for DynamicImage
{
    fn sample(&self, uv: Point<F>) -> F
    {
        let (w, h) = Sampler::<F, F>::dimensions(self);
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % w;
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % h;
        self.raw_sample(point!(x, y))
    }

    fn raw_sample(&self, uv: Point<u32>) -> F
    {
        let (w, h) = Sampler::<F, F>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        F::from_f32(c[0].to_f32().unwrap()) / max
    }

    fn dimensions(&self) -> (u32, u32) {
        GenericImageView::dimensions(self)
    }
}
