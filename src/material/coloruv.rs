use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorUV<F: Float> {
    scale: F,
}

impl<F: Float> ColorUV<F> {
    pub fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material for ColorUV<F> {
    type F = F;

    fn render(
        &self,
        maxel: &mut Maxel<F>,
        _lights: &[&dyn Light<F>],
        _rt: &dyn RayTracer<F>,
    ) -> Color<F> {
        let uv = maxel.uv();
        Color::new(uv.x, F::ZERO, uv.y) * self.scale
    }
}
