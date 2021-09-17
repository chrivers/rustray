use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Mirror<F: Float>
{
    refl: F
}

impl<F: Float> Mirror<F>
{
    pub fn new(refl: F) -> Self
    {
        Self { refl }
    }

}

impl<F: Float> Material for Mirror<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let refl = hit.dir.reflect(&maxel.normal);
        let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);
        c_refl * self.refl
    }
}
