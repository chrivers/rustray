use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct ColorUV<F: Float>
{
    scale: F,
}

impl<F: Float> ColorUV<F>
{
    pub fn new(scale: F) -> Self
    {
        Self { scale }
    }

}

impl<F: Float> Material for ColorUV<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        Color::new(self.scale * maxel.uv.x, self.scale * maxel.uv.y, F::zero())
    }
}
