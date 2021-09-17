use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Phong<F: Float, M: Material>
{
    mat: M,
    pow: F,
}

impl<F: Float, M: Material> Phong<F, M>
{
    pub fn new(pow: F, mat: M) -> Self
    {
        Self { pow, mat }
    }

}

impl<F: Float> Phong<F, Color<F>>
{
    pub fn white() -> Self
    {
        Self::new(F::from_u32(8), Color::white())
    }

    pub fn black() -> Self
    {
        Self::new(F::from_u32(8), Color::black())
    }
}

impl<F: Float, M: Material<F=F>> Material for Phong<F, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let mut res = Color::black();
        for light in lights {
            if rt.ray_shadow(hit, light) {
                continue
            }

            let m = hit.pos.vector_to(light.pos);
            let light_dir = m.normalized();
            let refl_dir = light_dir.reflect(&maxel.normal);
            let spec_angle = refl_dir.dot(hit.dir).max(F::zero());

            let self_color = self.mat.render(hit, maxel, lights, rt, lvl);
            let light_color = light.color * self_color / m.length();

            let lambert = maxel.normal.cos_angle(m);
            res += light_color * lambert;

            let specular = spec_angle.pow(self.pow);
            res += light_color * specular;
        }
        res
    }
}