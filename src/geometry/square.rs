use super::geo_util::*;

#[derive(Debug)]
pub struct Square<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Square<F, M>);

impl<F: Float, M: Material<F>> Interactive for Square<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                self.mat.ui(ui);
            });
    }
}

impl<F: Float, M: Material<F>> SceneObject for Square<F, M> {
    fn get_name(&self) -> &str {
        "Square"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Square<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        if r.dir.z.is_zero() {
            return None;
        }

        let t = -r.pos.z / r.dir.z;

        if t <= F::BIAS2 {
            return None;
        }

        let mut p = r.extend(t);
        p.x += F::HALF;
        p.y += F::HALF;

        if !p.x.is_unit() {
            return None;
        }

        if !p.y.is_unit() {
            return None;
        }

        let normal = if r.dir.z.is_positive() {
            -Vector::unit_z()
        } else {
            Vector::unit_z()
        };

        Some(
            ray.hit_at(t, self, &self.mat)
                .with_normal(self.xfrm.nml_inv(normal))
                .with_uv(point!(p.x, p.y)),
        )
    }
}

impl<F: Float, M: Material<F>> Square<F, M> {
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let xfrm = Transform::new(xfrm);
        let aabb = build_aabb_symmetric(&xfrm, F::HALF, F::HALF, F::ZERO);
        Self { xfrm, mat, aabb }
    }
}
