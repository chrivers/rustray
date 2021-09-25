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

    fn render(&self, _hit: &Hit<F>, maxel: &Maxel<F>, _lights: &[Light<F>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        Color::new(
            self.scale * maxel.normal.x,
            self.scale * maxel.normal.y,
            self.scale * maxel.normal.z,
        )
    }
}
