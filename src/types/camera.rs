use cgmath::{Angle, Deg, EuclideanSpace, InnerSpace, Matrix4, MetricSpace, Point3};

#[cfg(feature = "gui")]
use crate::frontend::gui::position_ui;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, Point, Ray, Transform, Vector};
use crate::vec3;

#[derive(Clone, Copy, Debug)]
pub struct Camera<F: Float> {
    pub model: Transform<F>,
    pub projection: Transform<F>,
    pub ndc: Transform<F>,
    pos: Vector<F>,
    dir: Vector<F>,
    xres: u32,
    yres: u32,
}

impl<F: Float> Camera<F> {
    pub fn parametric(
        pos: Vector<F>,
        lookat: Vector<F>,
        updir: Vector<F>,
        fov: F,
        xres: u32,
        yres: u32,
    ) -> Self {
        Self::build(pos, lookat - pos, updir, fov, xres, yres, None)
    }

    pub fn build(
        pos: Vector<F>,
        viewdir: Vector<F>,
        updir: Vector<F>,
        fov: F,
        xres: u32,
        yres: u32,
        aspect: Option<F>,
    ) -> Self {
        let dir = viewdir.normalize();
        let u = dir.cross(updir).normalize();
        let v = u.cross(dir).normalize();
        let aspect_ratio = aspect.unwrap_or_else(|| F::from_u32(xres) / F::from_u32(yres));
        let viewplane_height = Deg(fov / F::TWO).tan() * F::TWO;
        let viewplane_width = viewplane_height / aspect_ratio;
        let x_inc_vector = u * viewplane_width;
        let y_inc_vector = v * viewplane_height;
        info!("aspect_ratio: {}", aspect_ratio);
        info!("vp_width: {:.4}", viewplane_width);
        info!("vp_height: {:.4}", viewplane_height);
        info!("x_inc_vector: {:8.4?}", x_inc_vector);
        info!("y_inc_vector: {:8.4?}", y_inc_vector);

        let mat1 = cgmath::perspective(
            Deg(fov),
            aspect_ratio,
            F::from_f32(1.0),
            F::from_u32(10_000),
        );

        let mat2 = Matrix4::from_translation(vec3![F::HALF, F::HALF, F::ZERO]);
        let mat3 = Matrix4::from_nonuniform_scale(F::HALF, -F::HALF, F::ONE);

        let model = Transform::new(Matrix4::look_to_rh(Point3::from_vec(pos), viewdir, updir));
        let projection = Transform::new(mat1);
        let ndc = Transform::new(mat2 * mat3);

        Self {
            model,
            projection,
            ndc,
            pos,
            dir,
            xres,
            yres,
        }
    }

    pub fn get_ray(self, point: Point<F>, lvl: u32) -> Ray<F> {
        let pos = self.model.pos_inv(vec3![F::ZERO, F::ZERO, F::ZERO]);

        let vpp = self
            .model
            .dir_inv(
                self.projection
                    .pos_inv(self.ndc.pos_inv(vec3![point.x, point.y, F::ONE])),
            );

        Ray::new(pos, vpp.normalize(), lvl, false)
    }

    pub const fn size(self) -> (u32, u32) {
        (self.xres, self.yres)
    }

    pub fn distance2(self, pos: Vector<F>) -> F {
        self.pos.distance2(pos)
    }
}

impl<F: Float> Interactive<F> for Camera<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(format!("X resolution: {}", self.xres));
                        ui.end_row();

                        ui.label(format!("Y resolution: {}", self.yres));
                        ui.end_row();

                        position_ui(ui, &mut self.pos, "Position");
                        position_ui(ui, &mut self.dir, "Direction");
                        self.dir = self.dir.normalize();
                    })
            });
    }
}

impl<F: Float> SceneObject<F> for Camera<F> {
    fn get_name(&self) -> &str {
        "Camera"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}
