use super::geo_util::*;

#[derive(Debug)]
pub struct Sphere<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Sphere<F, M>);

impl<F: Float, M: Material<F>> Interactive<F> for Sphere<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.mat.ui(ui)
    }

    #[cfg(feature = "gui")]
    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        gizmo_ui(ui, camera, self, rect)
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for Sphere<F, M> {
    fn get_name(&self) -> &str {
        "Sphere"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> HasTransform<F> for Sphere<F, M> {
    fn get_transform(&self) -> &Transform<F> {
        &self.xfrm
    }

    fn set_transform(&mut self, xfrm: &Transform<F>) {
        self.xfrm = *xfrm;
        self.recompute_aabb();
    }
}

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Sphere<F, M> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::ONE, F::ONE, F::ONE);
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Sphere<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let result = r.intersect_unit_sphere()?;
        let normal = r.extend(result);

        let nml = self.xfrm.nml(normal);
        Some(
            ray.hit_at(result, self, &self.mat)
                .with_normal(nml.normalize())
                .with_uv(nml.polar_uv().into()),
        )
    }
}

impl<F: Float, M: Material<F>> Sphere<F, M> {
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }

    pub fn place(pos: Vector<F>, radius: F, mat: M) -> Self {
        let scale = Matrix4::from_scale(radius);
        let xlate = Matrix4::from_translation(pos);
        Self::new(xlate * scale, mat)
    }
}
