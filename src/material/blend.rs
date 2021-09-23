use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Blend<F: Float, A: Material, B: Material>
{
    a: A,
    b: B,
    pct: F,
}

impl<F: Float, A: Material, B: Material> Blend<F, A, B>
{
    pub fn new(a: A, b: B, pct: F) -> Self
    {
        Self { a, b, pct }
    }

}

impl<F: Float, A: Material<F=F>, B: Material<F=F>> Material for Blend<F, A, B>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let a = self.a.render(hit, maxel, light, rt);
        let b = self.b.render(hit, maxel, light, rt);
        a.lerp(b, self.pct)
    }
}
