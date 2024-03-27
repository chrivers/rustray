use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorPos<F: Float> {
    scale: F,
}

impl<F: Float> ColorPos<F> {
    pub fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material for ColorPos<F> {
    type F = F;

    fn render(
        &self,
        maxel: &mut Maxel<F>,
        _rt: &dyn RayTracer<F>,
    ) -> Color<F> {
        let mut n = maxel.pos / F::from_f32(32.0);
        n.x += F::ONE;
        n.y += F::ONE;
        Color::new(n.x, n.y, n.z) * self.scale
    }
}
