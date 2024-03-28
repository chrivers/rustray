use super::vector::{InnerSpace, MetricSpace};
use super::Float;
use super::Point;
use super::Ray;
use super::Vector;
#[cfg(feature = "gui")]
use crate::frontend::gui::position_ui;
use crate::scene::{Interactive, SceneObject};
use cgmath::{Angle, Deg};

#[derive(Clone, Copy, Debug)]
pub struct Camera<F: Float> {
    pos: Vector<F>,
    dir: Vector<F>,
    hor: Vector<F>,
    ver: Vector<F>,
    xres: u32,
    yres: u32,
}

impl<F: Float> Camera<F> {
    pub fn raw(
        pos: Vector<F>,
        lookat: Vector<F>,
        hor: Vector<F>,
        ver: Vector<F>,
        xres: u32,
        yres: u32,
    ) -> Camera<F> {
        let dir = (lookat - pos).normalize();

        info!(
            "Camera::raw [ pos:{:?},  dir:{:?},  hor:{:?},  ver:{:?} ]",
            pos, dir, hor, ver
        );
        Camera {
            pos,
            dir,
            hor,
            ver,
            xres,
            yres,
        }
    }

    pub fn parametric(
        pos: Vector<F>,
        lookat: Vector<F>,
        updir: Vector<F>,
        fov: F,
        xres: u32,
        yres: u32,
    ) -> Camera<F> {
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
    ) -> Camera<F> {
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

        Camera {
            pos,
            dir,
            hor: x_inc_vector,
            ver: y_inc_vector,
            xres,
            yres,
        }
    }

    pub fn get_ray(self, point: Point<F>) -> Ray<F> {
        let x = point.x - F::HALF;
        let y = -point.y + F::HALF;
        let vpp = self.dir + (self.hor * x) + (self.ver * y);
        Ray::new(self.pos, vpp.normalize(), 0)
    }

    pub fn size(self) -> (u32, u32) {
        (self.xres, self.yres)
    }

    pub fn distance2(self, pos: Vector<F>) -> F {
        self.pos.distance2(pos)
    }
}

impl<F: Float> Interactive for Camera<F> {
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

impl<F: Float> SceneObject for Camera<F> {
    fn get_name(&self) -> &str {
        "Camera"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}
