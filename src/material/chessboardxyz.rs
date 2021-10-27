use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
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

    fn render(&self, maxel: &mut Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let x = maxel.pos.x.abs().fract() > F::HALF;
        let y = maxel.pos.y.abs().fract() > F::HALF;
        let z = maxel.pos.z.abs().fract() > F::HALF;

        if x^y^z {
            self.a.render(maxel, lights, rt)
        } else {
            self.b.render(maxel, lights, rt)
        }
    }
}
