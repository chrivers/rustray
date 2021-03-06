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

    fn render(&self, _hit: &Hit<F>, maxel: &Maxel<F>, _lights: &[&dyn Light<F>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        Color::new(self.scale * maxel.uv.x, F::ZERO, self.scale * maxel.uv.y)
    }
}
