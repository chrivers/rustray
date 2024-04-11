use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorUV<F: Float> {
    scale: F,
}

impl<F: Float> ColorUV<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material<F> for ColorUV<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        Color::new(uv.x, F::ZERO, uv.y) * self.scale
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for ColorUV<F> {}

impl<F: Float> SceneObject<F> for ColorUV<F> {
    sceneobject_impl_body!("ColorUV");
}
