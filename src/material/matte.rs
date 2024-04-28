use std::marker::PhantomData;

use cgmath::InnerSpace;
use rand::Rng;

use crate::light::Lixel;
use crate::material::Material;
use crate::sampler::{Sampler, Texel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Vectorx};

#[derive(Copy, Clone, Debug)]
pub struct Matte<F: Float + Texel, S: Sampler<F, F>, M: Material<F>> {
    src: S,    /* Surface Roughness Coefficient */
    rays: u32, /* Number of rays to average over */
    mat: M,    /* Underlying material */
    _p: PhantomData<F>,
}

impl<F, S, M> Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F>,
{
    pub const fn new(src: S, rays: u32, mat: M) -> Self {
        Self {
            src,
            rays,
            mat,
            _p: PhantomData,
        }
    }
}

impl<F, S, M> Material<F> for Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F>,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let mut rng = rand::thread_rng();
        let mut col = Color::BLACK;
        let mut mxl = *maxel;

        let uv = maxel.uv();
        let normal = mxl.nml();
        let src = self.src.sample(uv);
        for _n in 0..self.rays {
            let rx = (rng.gen() - F::HALF) * src;
            let ry = (rng.gen() - F::HALF) * src;
            let rz = (rng.gen() / F::TWO) * (F::ONE - src) + src;
            let (normalu, normalv) = normal.surface_tangents();
            mxl = mxl.with_normal((normal * rz + normalu * rx + normalv * ry).normalize());

            col += self.mat.render(&mut mxl, rt);
        }
        col / F::from_u32(self.rays)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        self.mat.shadow(maxel, rt, lixel)
    }
}

impl<F, S, M> Interactive<F> for Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F>,
{
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;

        res |= ui
            .add(egui::Slider::new(&mut self.rays, 1..=32).text("Rays"))
            .changed();

        res |= self.src.ui(ui, "Surface Roughness Coefficient");
        res |= self.mat.ui(ui);

        res
    }
}

impl<F, S, M> SceneObject<F> for Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F>,
{
    sceneobject_impl_body!("Matte", egui_phosphor::regular::WAVES);
}
