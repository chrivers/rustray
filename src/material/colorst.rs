use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ColorST<F: Float> {
    scale: F,
}

impl<F: Float> ColorST<F> {
    pub const fn new(scale: F) -> Self {
        Self { scale }
    }
}

impl<F: Float> Material<F> for ColorST<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let st = maxel.st();
        let w = F::ONE - st.x - st.y;
        Color::new(st.x, w, st.y) * self.scale
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for ColorST<F> {}

impl<F: Float> SceneObject<F> for ColorST<F> {
    sceneobject_impl_body!("ColorST");
}
