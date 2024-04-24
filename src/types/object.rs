use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{FiniteGeometry, Geometry};
use crate::light::Lixel;
use crate::material::{DynMaterial, HasMaterial, Material};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::{Color, Float, Maxel, Point, Ray, Vector};

#[derive(Debug)]
pub struct NamedObject<S> {
    pub name: String,
    pub obj: S,
}

impl<S> NamedObject<S> {
    pub const fn new(name: String, obj: S) -> Self {
        Self { name, obj }
    }
}

impl<F: Float, T> Geometry<F> for NamedObject<T>
where
    T: Geometry<F>,
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        self.obj.intersect(ray)
    }

    fn normal(&self, maxel: &mut Maxel<F>) -> Vector<F> {
        self.obj.normal(maxel)
    }

    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F> {
        self.obj.uv(maxel)
    }

    fn st(&self, maxel: &mut Maxel<F>) -> Point<F> {
        self.obj.st(maxel)
    }
    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        self.obj.material()
    }
}

impl<T> rtbvh::Primitive for NamedObject<T>
where
    T: rtbvh::Primitive,
{
    fn center(&self) -> Vec3 {
        self.obj.center()
    }

    fn aabb(&self) -> Aabb {
        self.obj.aabb()
    }
}

impl<F: Float, T> FiniteGeometry<F> for NamedObject<T>
where
    T: FiniteGeometry<F>,
{
    fn recompute_aabb(&mut self) {
        self.obj.recompute_aabb();
    }
}

impl<F: Float, I: Interactive<F>> Interactive<F> for NamedObject<I> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.obj.ui(ui)
    }

    fn ui_center(
        &mut self,
        ui: &mut egui::Ui,
        camera: &crate::types::Camera<F>,
        rect: &egui::Rect,
    ) -> bool {
        self.obj.ui_center(ui, camera, rect)
    }

    fn ui_bounding_box(&mut self) -> Option<&rtbvh::Aabb> {
        self.obj.ui_bounding_box()
    }
}

impl<F: Float, M: Material<F>> Material<F> for NamedObject<M> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        self.obj.render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        self.obj.shadow(maxel, rt, lixel)
    }

    fn dynamic(self) -> DynMaterial<F>
    where
        Self: Sized + 'static,
    {
        self.obj.dynamic()
    }
}

impl<F: Float, S: SceneObject<F>> SceneObject<F> for NamedObject<S> {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_icon(&self) -> &str {
        self.obj.get_icon()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        self.obj.get_interactive()
    }

    fn get_id(&self) -> Option<usize> {
        self.obj.get_id()
    }
}
