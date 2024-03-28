use super::geo_util::*;

#[derive(Debug)]
pub struct Cube<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Cube<F, M>);

impl<F: Float, M: Material<F>> Interactive for Cube<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.mat.ui(ui);
    }
}

impl<F: Float, M: Material<F>> SceneObject for Cube<F, M> {
    fn get_name(&self) -> &str {
        "Cube"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Cube<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let p = r.pos;
        let d = r.dir;

        let mut best_t = F::max_value();
        let mut best = None;

        for it in 0..6 {
            let mod0 = it % 3;

            if d[mod0].is_zero() {
                continue;
            }

            let t = (F::from_usize(it / 3) - F::HALF - p[mod0]) / d[mod0];

            if t < F::BIAS2 || t > best_t {
                continue;
            }

            let mod1 = (it + 1) % 3;
            let mod2 = (it + 2) % 3;
            let x = p[mod1] + t * d[mod1];
            let y = p[mod2] + t * d[mod2];

            let half = -F::HALF..F::HALF;
            if half.contains(&x) && half.contains(&y) && best_t > t {
                best_t = t;
                best = Some(it);
            }
        }

        let best = best?;

        let normals = [Vector::unit_x(), Vector::unit_y(), Vector::unit_z()];

        let normal = if best < 3 {
            -normals[best % 3]
        } else {
            normals[best % 3]
        };

        let i1 = (best + 1) % 3;
        let i2 = (best + 2) % 3;
        let min = i1.min(i2);
        let max = i1.max(i2);

        let isec = r.extend(best_t);
        let uv = point!(F::HALF - isec[min], F::HALF - isec[max]);

        Some(
            ray.hit_at(best_t, self, &self.mat)
                .with_normal(self.xfrm.nml(normal))
                .with_uv(uv),
        )
    }
}

impl<F: Float, M: Material<F>> Cube<F, M> {
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let xfrm = Transform::new(xfrm);
        let aabb = build_aabb_symmetric(&xfrm, F::HALF, F::HALF, F::HALF);
        Self { xfrm, mat, aabb }
    }
}
