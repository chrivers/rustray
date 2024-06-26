use crate::light::Lixel;
use crate::material::Material;
use crate::point;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Point};

/// Proxy material that scales UV coordinates, before rendering backing material.
#[derive(Copy, Clone, Debug)]
pub struct ScaleUV<F: Float, M: Material<F>> {
    uv: Point<F>,
    mat: M,
}

impl<F: Float, M: Material<F>> ScaleUV<F, M> {
    pub fn new(u: F, v: F, mat: M) -> Self {
        Self {
            uv: point!(u, v),
            mat,
        }
    }
}

impl<F: Float, M: Material<F>> Material<F> for ScaleUV<F, M> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let mut smaxel = maxel.with_uv(self.uv.dot(uv));
        self.mat.render(&mut smaxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        let uv = maxel.uv();
        let mut smaxel = maxel.with_uv(self.uv.dot(uv));
        self.mat.shadow(&mut smaxel, rt, lixel)
    }
}

impl<F: Float, M: Material<F>> Interactive<F> for ScaleUV<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;

        res |= ui
            .add(
                egui::Slider::new(&mut self.uv.x, F::ZERO..=F::from_u32(50))
                    .logarithmic(true)
                    .clamp_to_range(false)
                    .trailing_fill(true)
                    .text("u scaling factor"),
            )
            .changed();
        res |= ui
            .add(
                egui::Slider::new(&mut self.uv.y, F::ZERO..=F::from_u32(50))
                    .logarithmic(true)
                    .clamp_to_range(false)
                    .trailing_fill(true)
                    .text("v scaling factor"),
            )
            .changed();
        res |= self.mat.ui(ui);

        res
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for ScaleUV<F, M> {
    sceneobject_impl_body!("Scale UV", egui_phosphor::regular::FRAME_CORNERS);
}
