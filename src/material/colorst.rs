use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorST<F: Float>
{
    scale: F,
}

impl<F: Float> ColorST<F>
{
    pub fn new(scale: F) -> Self
    {
        Self { scale }
    }
}

impl<F: Float> Material for ColorST<F>
{
    type F = F;

    fn render(&self, _hit: &Hit<F>, maxel: &Maxel<F>, _lights: &[&dyn Light<F>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        let st = maxel.st;
        let w = F::ONE - st.x - st.y;
        Color::new(st.x, w, st.y) * self.scale
    }
}