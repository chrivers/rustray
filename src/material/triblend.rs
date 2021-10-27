use super::mat_util::*;

#[derive(Clone, Debug)]
pub struct Triblend<F: Float, A: Material, B: Material, C: Material>
{
    a: A,
    b: B,
    c: C,
    p: PhantomData<F>,
}

impl<F: Float, A: Material, B: Material, C: Material> Triblend<F, A, B, C>
{
    pub fn new(a: A, b: B, c: C) -> Self
    {
        Self { a, b, c, p: PhantomData { } }
    }
}

impl<F: Float, A: Material<F=F>, B: Material<F=F>, C: Material<F=F>> Material for Triblend<F, A, B, C>
{
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let a = self.a.render(maxel, lights, rt);
        let b = self.b.render(maxel, lights, rt);
        let c = self.c.render(maxel, lights, rt);

        let st = maxel.st();
        let u = st.x;
        let v = st.y;
        let w = F::ONE - u - v;

        (a * w) + (b * u) + (c * v)
    }
}
