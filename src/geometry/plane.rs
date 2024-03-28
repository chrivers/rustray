use super::geo_util::*;

#[derive(Clone, Copy, Debug)]
pub struct Plane<F: Float, M: Material<F = F>> {
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
    u: Vector<F>,
    v: Vector<F>,
    mat: M,
}

impl<F: Float, M: Material<F = F>> Interactive for Plane<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                position_ui(ui, &mut self.pos, "Position");
                self.mat.ui(ui);
            });
    }
}

impl<F: Float, M: Material<F = F>> SceneObject for Plane<F, M> {
    fn get_name(&self) -> &str {
        "Plane"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F = F>> Geometry<F> for Plane<F, M> {
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
}

impl<F: Float, M: Material<F = F>> Plane<F, M> {
    pub fn new(pos: Vector<F>, d1: Vector<F>, d2: Vector<F>, mat: M) -> Self {
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
