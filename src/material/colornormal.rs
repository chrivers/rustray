use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct ColorNormal<F: Float>
{
    scale: F,
}

impl<F: Float> ColorNormal<F>
{
    pub fn new(scale: F) -> Self
    {
        Self { scale }
    }

}

impl<F: Float> Material for ColorNormal<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        Color::new(self.scale * maxel.uv.x, self.scale * maxel.uv.y, F::zero())
    }
}
