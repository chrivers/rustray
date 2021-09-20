use image::{GenericImageView, Pixel};
use num_traits::ToPrimitive;
use image::DynamicImage;

use super::samp_util::*;

#[derive(Copy, Clone)]
/**
`Texture3` implements [`Sampler<F, F>`] by looking up values in a texture
*/
pub struct Texture3<I: GenericImageView + Sync>
{
    /** Backing image */
    img: I,
}

impl<I: GenericImageView + Sync> Texture3<I>
{
    pub fn new(img: I) -> Self
    {
        Self { img }
    }
}

impl<F: Float, I: GenericImageView + Sync> Sampler<F, Color<F>> for Texture3<I>
{
    fn sample(&self, uv: Point<F>) -> Color<F>
    {
        let (w, h) = self.img.dimensions();
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % w;
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % h;
        self.raw_sample(point!(x, y))
    }

    fn raw_sample(&self, uv: Point<u32>) -> Color<F>
    {
        let (w, h) = self.img.dimensions();
        let c = self.img.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        Color::new(
            F::from_f32(c[0].to_f32().unwrap()) / max,
            F::from_f32(c[1].to_f32().unwrap()) / max,
            F::from_f32(c[2].to_f32().unwrap()) / max,
        )
    }

    fn dimensions(&self) -> (u32, u32) {
        self.img.dimensions()
    }
}

impl<F: Float> Sampler<F, Color<F>> for DynamicImage
{
    fn sample(&self, uv: Point<F>) -> Color<F>
    {
        let (w, h) = Sampler::<F, Color<F>>::dimensions(self);
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % w;
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % h;
        self.raw_sample(point!(x, y))
    }

    fn raw_sample(&self, uv: Point<u32>) -> Color<F>
    {
        let (w, h) = Sampler::<F, Color<F>>::dimensions(self);
        let c = self.get_pixel(uv.x % w, uv.y % h).to_rgb();
        let max = F::from_u32(0xFF);
        Color::new(
            F::from_f32(c[0].to_f32().unwrap()) / max,
            F::from_f32(c[1].to_f32().unwrap()) / max,
            F::from_f32(c[2].to_f32().unwrap()) / max,
        )
    }

    fn dimensions(&self) -> (u32, u32) {
        GenericImageView::dimensions(self)
    }
}
