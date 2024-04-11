use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ChessBoard<F: Float, A: Material<F>, B: Material<F>> {
    a: A,
    b: B,
    _p: PhantomData<F>,
}

impl<F: Float, A: Material<F>, B: Material<F>> ChessBoard<F, A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            _p: PhantomData,
        }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>> Material<F> for ChessBoard<F, A, B> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let u = uv.x.abs().fract() > F::HALF;
        let v = uv.y.abs().fract() > F::HALF;

        if u ^ v {
            self.a.render(maxel, rt)
        } else {
            self.b.render(maxel, rt)
        }
    }
}

#[cfg(feature = "gui")]
impl<F: Float, A: Material<F>, B: Material<F>> Interactive<F> for ChessBoard<F, A, B> {}

impl<F: Float, A: Material<F>, B: Material<F>> SceneObject<F> for ChessBoard<F, A, B> {
    sceneobject_impl_body!("Chessboard");
}
