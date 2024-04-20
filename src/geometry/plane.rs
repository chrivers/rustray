use cgmath::InnerSpace;

use crate::geometry::Geometry;
use crate::material::HasMaterial;
use crate::point;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, MaterialId, Maxel, Point, Ray, Vector};

#[derive(Clone, Copy, Debug)]
pub struct Plane<F: Float> {
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
    u: Vector<F>,
    v: Vector<F>,
    mat: MaterialId,
}

impl<F: Float> Interactive<F> for Plane<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::gui::controls;

        let mut res = false;
        res |= controls::position(ui, &mut self.pos, "Position");
        res |= Interactive::<F>::ui(&mut self.mat, ui);
        res
    }
}

geometry_impl_sceneobject!(Plane<F>, "Plane");
geometry_impl_hasmaterial!(Plane<F>);

impl<F: Float> Geometry<F> for Plane<F> {
    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F> {
        let u = self.u.dot(maxel.pos);
        let v = self.v.dot(maxel.pos);
        point!(u, v)
    }

    fn normal(&self, _hit: &mut Maxel<F>) -> Vector<F> {
        self.normal
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, self, &self.mat))
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }
}

impl<F: Float> Plane<F> {
    const ICON: &'static str = egui_phosphor::regular::SQUARE_LOGO;

    pub fn new(pos: Vector<F>, d1: Vector<F>, d2: Vector<F>, mat: MaterialId) -> Self {
        let dir1 = d1.normalize();
        let dir2 = d2.normalize();
        let normal = dir1.cross(dir2);

        Self {
            pos,
            dir1,
            dir2,
            normal,
            u: d1,
            v: d2,
            mat,
        }
    }
}
