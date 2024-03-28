use super::geo_util::*;

#[derive(Debug)]
pub struct Sphere<F: Float, M: Material<F = F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Sphere<F, M>);

impl<F: Float, M: Material<F = F>> Interactive for Sphere<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.mat.ui(ui);
    }
}

impl<F: Float, M: Material<F = F>> SceneObject for Sphere<F, M> {
    fn get_name(&self) -> &str {
        "Sphere"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F = F>> Geometry<F> for Sphere<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let result = r.intersect_sphere(&Vector::zero(), F::ONE)?;
        let normal = r.extend(result);

        let nml = self.xfrm.nml(normal);
        Some(
            ray.hit_at(result, self, &self.mat)
                .with_normal(nml)
                .with_uv(nml.polar_uv().into()),
        )
    }
}

impl<F: Float, M: Material<F = F>> Sphere<F, M> {
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let xfrm = Transform::new(xfrm);
        let aabb = build_aabb_symmetric(&xfrm, F::ONE, F::ONE, F::ONE);
        Self { xfrm, mat, aabb }
    }

    pub fn place(pos: Vector<F>, radius: F, mat: M) -> Self {
        let scale = Matrix4::from_scale(radius);
        let xlate = Matrix4::from_translation(pos);
        Self::new(xlate * scale, mat)
    }
}
