use super::mat_util::*;

/// Material blender, that interpolates between three materials.
///
/// This material composes three materials (A, B, and C), and blends linearly
/// between these, based on UV coordinates.
///
/// Useful for representing triangles with heterogenous materials.

#[derive(Clone, Debug)]
pub struct Triblend<F: Float, A: Material<F>, B: Material<F>, C: Material<F>> {
    a: A,
    b: B,
    c: C,
    p: PhantomData<F>,
}

impl<F: Float, A: Material<F>, B: Material<F>, C: Material<F>> Triblend<F, A, B, C> {
    pub const fn new(a: A, b: B, c: C) -> Self {
        Self {
            a,
            b,
            c,
            p: PhantomData {},
        }
    }
}

impl<F: Float, A: Material<F>, B: Material<F>, C: Material<F>> Material<F>
    for Triblend<F, A, B, C>
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let a = self.a.render(maxel, rt);
        let b = self.b.render(maxel, rt);
        let c = self.c.render(maxel, rt);

        let st = maxel.st();
        let u = st.x;
        let v = st.y;
        let w = F::ONE - u - v;

        (a * w) + (b * u) + (c * v)
    }
}

#[cfg(feature = "gui")]
impl<F, A, B, C> Interactive<F> for Triblend<F, A, B, C>
where
    F: Float,
    A: Material<F> + Interactive<F>,
    B: Material<F> + Interactive<F>,
    C: Material<F> + Interactive<F>,
{
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;

        res |= self.a.ui(ui);
        res |= self.b.ui(ui);
        res |= self.c.ui(ui);

        res
    }
}

#[cfg(feature = "gui")]
impl<F, A, B, C> SceneObject<F> for Triblend<F, A, B, C>
where
    F: Float,
    A: Material<F> + Interactive<F>,
    B: Material<F> + Interactive<F>,
    C: Material<F> + Interactive<F>,
{
    sceneobject_impl_body!("Triblend");
}
