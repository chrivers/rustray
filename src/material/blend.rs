use cgmath::VectorSpace;

use crate::light::Lixel;
use crate::material::Material;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel};

#[derive(Copy, Clone, Debug)]
pub struct Blend<F: Float, A: Material<F>, B: Material<F>> {
    a: A,
    b: B,
    pct: F,
}

impl<F: Float, A: Material<F>, B: Material<F>> Blend<F, A, B> {
    pub const fn new(a: A, b: B, pct: F) -> Self {
        Self { a, b, pct }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>> Material<F> for Blend<F, A, B> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let a = self.a.render(maxel, rt);
        let b = self.b.render(maxel, rt);
        a.lerp(b, self.pct)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        let a = self.a.shadow(maxel, rt, lixel);
        let b = self.b.shadow(maxel, rt, lixel);
        a.lerp(b, self.pct)
    }
}

#[cfg(feature = "gui")]
impl<F: Float, A: Material<F>, B: Material<F>> Interactive<F> for Blend<F, A, B> {}

#[cfg(feature = "gui")]
impl<F: Float, A: Material<F>, B: Material<F>> SceneObject<F> for Blend<F, A, B> {
    sceneobject_impl_body!("Blend");
}
