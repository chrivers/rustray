use super::geo_util::*;

#[derive(Debug)]
pub struct Cone<F: Float, M: Material<F = F>> {
    height: F,
    top_r: F,
    bot_r: F,
    capped: bool,
    mat: M,
    xfrm: Transform<F>,
    aabb: Aabb,
}

aabb_impl_fm!(Cone<F, M>);

impl<F: Float, M: Material<F = F>> Interactive for Cone<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.add(
                    Slider::new(&mut self.top_r, F::ZERO..=F::from_u32(10))
                        .clamp_to_range(false)
                        .smallest_positive(0.01)
                        .text("Top radius"),
                );
                ui.end_row();

                ui.add(
                    Slider::new(&mut self.bot_r, F::ZERO..=F::from_u32(10))
                        .clamp_to_range(false)
                        .smallest_positive(0.01)
                        .text("Bottom radius"),
                );
                ui.end_row();

                ui.add(
                    Slider::new(&mut self.height, F::ZERO..=F::from_u32(10))
                        .clamp_to_range(false)
                        .smallest_positive(0.01)
                        .text("Height"),
                );
                ui.end_row();

                ui.checkbox(&mut self.capped, "Capped");
                ui.end_row();

                /* position_ui(ui, &mut self.pos, "Position"); */
                self.mat.ui(ui);
            });
        let m = self.bot_r.max(self.top_r);
        self.aabb = build_aabb_ranged(&self.xfrm, [-m, m], [-m, m], [F::ZERO, self.height]);
    }
}

impl<F: Float, M: Material<F = F>> SceneObject for Cone<F, M> {
    fn get_name(&self) -> &str {
        "Cone"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F = F>> Geometry<F> for Cone<F, M> {
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cone.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let bot_r = self.bot_r.abs().max(F::BIAS);
        let top_r = self.top_r.abs().max(F::BIAS);

        let mut beta = (top_r - bot_r) / self.height;

        if beta.abs() < F::BIAS {
            beta = F::BIAS;
        }

        let mut gamma;
        gamma = if beta.is_negative() {
            top_r / beta
        } else {
            bot_r / beta
        };

        if gamma.is_negative() {
            gamma -= self.height;
        }

        let mut normal: Vector<F> = Vector::unit_x();

        let p = r.pos;
        let d = r.dir;

        let beta2 = beta * beta;

        let pzg = p.z + gamma;

        let a = d.x * d.x + d.y * d.y - beta2 * d.z * d.z;
        let b = F::TWO * (p.x * d.x + p.y * d.y - beta2 * pzg * d.z);
        let c = p.x * p.x + p.y * p.y - beta2 * pzg * pzg;

        fn test_cap<F: Float>(
            root: &mut F,
            normal: &mut Vector<F>,
            tx: F,
            r: &Ray<F>,
            rad: F,
            dz: F,
        ) {
            if tx >= *root || tx <= F::ZERO {
                return;
            }

            let p = r.extend(tx);
            if p.x * p.x + p.y * p.y <= rad * rad {
                *root = tx;
                if dz.is_positive() {
                    *normal = -Vector::unit_z();
                } else {
                    *normal = Vector::unit_z();
                }
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn test_side<F: Float>(
            root: &mut F,
            normal: &mut Vector<F>,
            tx: F,
            func: impl Fn(F, F) -> bool,
            r: &Ray<F>,
            height: F,
            beta2: F,
            gamma: F,
        ) {
            let point = r.extend(tx);
            let good = point.z >= F::ZERO && point.z <= height;
            if good && func(tx, *root) {
                *root = tx;
                *normal = vec3!(point.x, point.y, -F::TWO * beta2 * (point.z + gamma));
            }
        }

        let mut root = F::BIAS;

        let (root2, root1) = crate::types::ray::quadratic2(a, b, c)?;

        test_side(
            &mut root,
            &mut normal,
            root1,
            |tx, root| (tx > root) && (tx > F::BIAS),
            &r,
            self.height,
            beta2,
            gamma,
        );

        test_side(
            &mut root,
            &mut normal,
            root2,
            |tx, root| (tx < root) || (tx > F::BIAS),
            &r,
            self.height,
            beta2,
            gamma,
        );

        if self.capped {
            /* These are to help with finding caps */
            let t1 = (-p.z) / d.z;
            let t2 = (self.height - p.z) / d.z;

            test_cap(&mut root, &mut normal, t1, &r, self.bot_r, d.z);
            test_cap(&mut root, &mut normal, t2, &r, self.top_r, d.z);
        } else if normal.dot(r.dir) > top_r * bot_r {
            /* In case we are _inside_ the _uncapped_ cone, we need to flip the normal. */
            /* Essentially, the cone in this case is a double-sided surface */
            /* and has _2_ normals */
            normal = -normal;
        }

        if root <= F::BIAS {
            return None;
        }

        Some(
            ray.hit_at(root, self, &self.mat)
                .with_normal(self.xfrm.nml_inv(normal)),
        )
    }
}

impl<F: Float, M: Material<F = F>> Cone<F, M> {
    pub fn new(
        height: F,
        top_r: F,
        bot_r: F,
        capped: bool,
        xfrm: Matrix4<F>,
        mat: M,
    ) -> Cone<F, M> {
        let m = bot_r.max(top_r);
        let xfrm = Transform::new(xfrm);
        let aabb = build_aabb_ranged(&xfrm, [-m, m], [-m, m], [F::ZERO, height]);
        Cone {
            height,
            top_r,
            bot_r,
            capped,
            mat,
            xfrm,
            aabb,
        }
    }
}
