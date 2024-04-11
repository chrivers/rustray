use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorNormal<F: Float> {
    scale: F,
}

impl<F: Float> ColorNormal<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material<F> for ColorNormal<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let n = maxel.nml();
        Color::new(n.x.abs(), n.y.abs(), n.z.abs()) * self.scale
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for ColorNormal<F> {}

impl<F: Float> SceneObject<F> for ColorNormal<F> {
    sceneobject_impl_body!("Color Normal");
}
