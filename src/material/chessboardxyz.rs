use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ChessBoardXYZ<F: Float, A: Material<F>, B: Material<F>> {
    a: A,
    b: B,
    _p: PhantomData<F>,
}

impl<F: Float, A: Material<F>, B: Material<F>> ChessBoardXYZ<F, A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            _p: PhantomData,
        }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>> Material<F> for ChessBoardXYZ<F, A, B> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let x = maxel.pos.x.abs().fract() > F::HALF;
        let y = maxel.pos.y.abs().fract() > F::HALF;
        let z = maxel.pos.z.abs().fract() > F::HALF;

        if x ^ y ^ z {
            self.a.render(maxel, rt)
        } else {
            self.b.render(maxel, rt)
        }
    }
}

#[cfg(feature = "gui")]
impl<F: Float, A: Material<F>, B: Material<F>> Interactive<F> for ChessBoardXYZ<F, A, B> {}

impl<F: Float, A: Material<F>, B: Material<F>> SceneObject<F> for ChessBoardXYZ<F, A, B> {
    sceneobject_impl_body!("ChessBoardXYZ");
}
