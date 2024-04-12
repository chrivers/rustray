use std::marker::PhantomData;

use crate::light::Lixel;
use crate::material::Material;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel};

#[derive(Copy, Clone, Debug)]
pub enum ChessBoardMode {
    UV,
    XYZ,
}

#[derive(Copy, Clone, Debug)]
pub struct ChessBoard<F: Float, A: Material<F>, B: Material<F>> {
    a: A,
    b: B,
    mode: ChessBoardMode,
    _p: PhantomData<F>,
}

impl<F: Float, A: Material<F>, B: Material<F>> ChessBoard<F, A, B> {
    pub const fn new(mode: ChessBoardMode, a: A, b: B) -> Self {
        Self {
            a,
            b,
            mode,
            _p: PhantomData,
        }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>> ChessBoard<F, A, B> {
    fn select(&self, maxel: &mut Maxel<F>) -> bool {
        match self.mode {
            ChessBoardMode::UV => {
                let uv = maxel.uv();
                let u = uv.x.abs().fract() > F::HALF;
                let v = uv.y.abs().fract() > F::HALF;
                u ^ v
            }
            ChessBoardMode::XYZ => {
                let x = maxel.pos.x.abs().fract() > F::HALF;
                let y = maxel.pos.y.abs().fract() > F::HALF;
                let z = maxel.pos.z.abs().fract() > F::HALF;
                x ^ y ^ z
            }
        }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>> Material<F> for ChessBoard<F, A, B> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        if self.select(maxel) {
            self.a.render(maxel, rt)
        } else {
            self.b.render(maxel, rt)
        }
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        if self.select(maxel) {
            self.a.shadow(maxel, rt, lixel)
        } else {
            self.b.shadow(maxel, rt, lixel)
        }
    }
}

#[cfg(feature = "gui")]
impl<F: Float, A: Material<F>, B: Material<F>> Interactive<F> for ChessBoard<F, A, B> {}

impl<F: Float, A: Material<F>, B: Material<F>> SceneObject<F> for ChessBoard<F, A, B> {
    sceneobject_impl_body!("Chessboard");
}
