use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ChessBoard<A: Material, B: Material>
{
    a: A,
    b: B,
}

impl<A: Material, B: Material> ChessBoard<A, B>
{
    pub fn new(a: A, b: B) -> Self
    {
        Self { a, b }
    }

}

impl<F: Float, A: Material<F=F>, B: Material<F=F>> Material for ChessBoard<A, B>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let u = maxel.uv.x.abs().fract() > F::HALF;
        let v = maxel.uv.y.abs().fract() > F::HALF;

        if u^v {
            self.a.render(hit, maxel, light, rt)
        } else {
            self.b.render(hit, maxel, light, rt)
        }
    }
}
