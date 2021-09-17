use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct ChessBoardXYZ<A: Material, B: Material>
{
    a: A,
    b: B,
}

impl<A: Material, B: Material> ChessBoardXYZ<A, B>
{
    pub fn new(a: A, b: B) -> Self
    {
        Self { a, b }
    }

}

impl<F: Float, A: Material<F=F>, B: Material<F=F>> Material for ChessBoardXYZ<A, B>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let x = hit.pos.x.abs().fract() > F::HALF;
        let y = hit.pos.y.abs().fract() > F::HALF;
        let z = hit.pos.z.abs().fract() > F::HALF;

        if x^y^z {
            self.a.render(hit, maxel, light, rt, lvl)
        } else {
            self.b.render(hit, maxel, light, rt, lvl)
        }
    }
}
