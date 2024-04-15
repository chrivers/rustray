use cgmath::{Angle, Deg, EuclideanSpace, InnerSpace, Matrix4, MetricSpace, Point3};

use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, Point, Ray, Transform, Vector};
use crate::{sceneobject_impl_body, vec3};

#[derive(Clone, Copy, Debug)]
pub struct Camera<F: Float> {
    pub model: Transform<F>,
    pub projection: Transform<F>,
    pub ndc: Transform<F>,
    pos: Vector<F>,
    #[allow(dead_code)]
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

    pub fn get_ray(self, point: Point<F>) -> Ray<F> {
        let pos = self.model.pos_inv(vec3![F::ZERO, F::ZERO, F::ZERO]);

        let vpp = self
            .model
            .dir_inv(
                self.projection
                    .pos_inv(self.ndc.pos_inv(vec3![point.x, point.y, F::ONE])),
            );

        Ray::new(pos, vpp.normalize())
    }

    pub const fn size(self) -> (u32, u32) {
        (self.xres, self.yres)
    }

    pub fn distance2(self, pos: Vector<F>) -> F {
        self.pos.distance2(pos)
    }

    pub fn world_to_ndc(&self, pos: Vector<F>) -> Vector<F> {
        self.ndc.pos(self.projection.pos(self.model.pos(pos)))
    }
}

impl<F: Float> Interactive<F> for Camera<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::gui::controls;

        let mut res = false;

        ui.label(format!("X resolution: {}", self.xres));
        ui.end_row();

        ui.label(format!("Y resolution: {}", self.yres));
        ui.end_row();

        res |= controls::position(ui, &mut self.pos, "Position");
        res |= controls::position(ui, &mut self.dir, "Direction");
        self.dir = self.dir.normalize();

        res
    }
}

impl<F: Float> SceneObject<F> for Camera<F> {
    sceneobject_impl_body!("Camera", egui_phosphor::regular::VIDEO_CAMERA);
}

#[cfg(test)]
mod test {
    use crate::types::{Camera, Point, Vector, Vectorx};
    use crate::{point, vec3};

    #[test]
    fn test_camera() {
        /* colog::init(); */
        let camera = Camera::build(
            vec3![0.0, -20.0, 0.0],
            -Vector::UNIT_Z,
            Vector::UNIT_Y,
            50.0,
            100,
            100,
            None,
        );
        for point in [
            point!(0.0, 0.0),
            point!(1.0, 0.0),
            point!(0.5, 0.5),
            point!(0.0, 1.0),
            point!(1.0, 1.0),
        ] {
            let ray = camera.get_ray(point);
            info!("Point [{point:?}] | {:7.4?}", ray.dir);

            /* let ray1 = camera.get_ray(point, 1); */
            /* let ray2 = camera.get_ray2(point, 1); */
            /* info!("Point [{point:?}] | {:7.4?} | {:7.4?}", ray1.dir, ray2.dir); */
        }
    }
}
