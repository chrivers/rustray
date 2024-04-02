use super::geo_util::*;

#[derive(Debug)]
pub struct Cylinder<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    capped: bool,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Cylinder<F, M>);

impl<F: Float, M: Material<F>> Interactive<F> for Cylinder<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.capped, "Capped");
                ui.end_row();

                self.mat.ui(ui);
            });
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for Cylinder<F, M> {
    fn get_name(&self) -> &str {
        "Cylinder"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> HasTransform<F> for Cylinder<F, M> {
    fn get_transform(&self) -> &Transform<F> {
        &self.xfrm
    }

    fn set_transform(&mut self, xfrm: &Transform<F>) {
        self.xfrm = *xfrm;
        self.recompute_aabb();
    }
}

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Cylinder<F, M> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_ranged(
            &self.xfrm,
            [-F::ONE, F::ONE],
            [-F::ONE, F::ONE],
            [F::ZERO, F::ONE],
        );
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Cylinder<F, M> {
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cylinder.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        fn isect_body<F: Float>(r: &Ray<F>, capped: bool) -> Option<(F, Vector<F>)> {
            let p = r.pos;
            let d = r.dir;

            let a = d.x * d.x + d.y * d.y;
            let b = F::TWO * (p.x * d.x + p.y * d.y);
            let c = p.x * p.x + p.y * p.y - F::ONE;

            let (t1, t2) = crate::types::ray::quadratic2(a, b, c)?;
            if t1.is_negative() || t2.is_negative() {
                return None;
            }

            fn isect_side<F: Float>(r: &Ray<F>, t: F, capped: bool) -> Option<(F, Vector<F>)> {
                let p = r.extend(t);

                if !p.z.is_unit() {
                    return None;
                }

                let mut normal = vec3!(p.x, p.y, F::ZERO);

                /* In case we are _inside_ the _uncapped_ cone, we need to flip the normal. */
                /* Essentially, the cone in this case is a double-sided surface */
                /* and has _2_ normals */
                if !capped && r.dir.dot(normal).is_positive() {
                    normal = -normal;
                }
                Some((t, normal))
            }

            isect_side(r, t1, capped).or_else(|| isect_side(r, t2, capped))
        }

        fn isect_caps<F: Float>(r: &Ray<F>) -> Option<(F, Vector<F>)> {
            let pz = r.pos.z;
            let dz = r.dir.z;

            if dz.is_zero() {
                return None;
            }

            let t1;
            let t2;

            if dz.is_positive() {
                t1 = (-pz) / dz;
                t2 = (F::ONE - pz) / dz;
            } else {
                t1 = (F::ONE - pz) / dz;
                t2 = (-pz) / dz;
            }

            if t1 < F::BIAS {
                return None;
            }

            let t = if t1 >= F::BIAS { t1 } else { t2 };

            let p = r.extend(t);
            if (p.x * p.x + p.y * p.y) <= F::ONE {
                let n = if dz.is_positive() {
                    /* Intersection with cap at z = 0. */
                    -Vector::unit_z()
                } else {
                    Vector::unit_z()
                };
                return Some((t, n));
            }
            None
        }

        let r = ray.xfrm_inv(&self.xfrm);
        let body = isect_body(&r, self.capped);

        if self.capped {
            if let Some((t1, n1)) = isect_caps(&r) {
                if let Some((t2, n2)) = body {
                    if t2 < t1 {
                        return Some(
                            ray.hit_at(t2, self, &self.mat)
                                .with_normal(self.xfrm.nml(n2)),
                        );
                    }
                }
                return Some(
                    ray.hit_at(t1, self, &self.mat)
                        .with_normal(self.xfrm.nml(n1)),
                );
            }
        }

        if let Some((t2, n2)) = body {
            Some(
                ray.hit_at(t2, self, &self.mat)
                    .with_normal(self.xfrm.nml(n2)),
            )
        } else {
            None
        }
    }
}

impl<F: Float, M: Material<F>> Cylinder<F, M> {
    pub fn new(xfrm: Matrix4<F>, capped: bool, mat: M) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            capped,
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}
