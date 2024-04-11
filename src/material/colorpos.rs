use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorPos<F: Float> {
    scale: F,
}

impl<F: Float> ColorPos<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material<F> for ColorPos<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let mut n = maxel.pos / F::from_f32(32.0);
        n.x += F::ONE;
        n.y += F::ONE;
        Color::new(n.x, n.y, n.z) * self.scale
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for ColorPos<F> {}

impl<F: Float> SceneObject<F> for ColorPos<F> {
    sceneobject_impl_body!("ColorPos");
}
