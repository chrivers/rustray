use cgmath::InnerSpace;

/* use rand::Rng; */

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::iter::GridSamples;
use crate::types::maxel::Maxel;
use crate::types::{Color, Float, Vector};
use crate::Vectorx;

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

use super::Attenuation;

#[derive(Debug)]
pub struct AreaLight<F: Float> {
    pub attn: Attenuation<F>,
    pos: Vector<F>,
    dir: Vector<F>,
    upd: Vector<F>,
    pub color: Color<F>,
    pub width: F,
    pub height: F,
    dir1: Vector<F>,
    dir2: Vector<F>,
    xres: u32,
    yres: u32,
}

impl<F: Float> AreaLight<F> {
    pub fn new(
        attn: Attenuation<F>,
        pos: Vector<F>,
        dir: Vector<F>,
        upd: Vector<F>,
        color: Color<F>,
        width: F,
        height: F,
    ) -> Self {
        let dir = dir.normalize();
        let upd = upd.normalize();
        let (dir1, dir2) = Self::compute_dirs(dir, upd);

        Self {
            attn,
            width,
            height,
            pos,
            dir,
            upd,
            dir1,
            dir2,
            color,
            xres: 8,
            yres: 8,
        }
    }

    fn compute_dirs(dir: Vector<F>, upd: Vector<F>) -> (Vector<F>, Vector<F>) {
        let dir1 = dir.cross(upd);
        let dir2 = dir.cross(dir1);
        (dir1, dir2)
    }
}

impl<F: Float> SceneObject<F> for AreaLight<F> {
    fn get_name(&self) -> &str {
        "Area Light"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float> Interactive<F> for AreaLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::frontend::gui::attenuation_ui;

        egui::CollapsingHeader::new("Area light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let mut res = false;

                        res |= color_ui(ui, &mut self.color, "Color");
                        res |= attenuation_ui(ui, &mut self.attn);
                        res |= position_ui(ui, &mut self.pos, "Position");
                        res |= position_ui(ui, &mut self.dir, "Direction");

                        if res {
                            let (dir1, dir2) = Self::compute_dirs(self.dir, self.upd);
                            self.dir1 = dir1;
                            self.dir2 = dir2;
                        }
                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}

impl<F: Float> Light<F> for AreaLight<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let mut color = Color::BLACK;

        /* let mut rng = rand::thread_rng(); */

        let dist = GridSamples::new(self.width, self.height, self.xres, self.yres);

        for (rx, ry) in dist.iter() {
            let pos = self.pos + self.dir1 * rx + self.dir2 * ry;

            let dir = maxel.pos.vector_to(pos);
            let len2 = dir.magnitude2();
            let len = len2.sqrt();
            let col = self.attn.attenuate(self.color, len, len2);

            let lixel = Lixel {
                color: col,
                dir: dir / len,
                len2,
            };

            color += rt.shadow(maxel, lixel).color;
        }

        color = color / F::from_u32(self.xres * self.yres);

        let dir = maxel.pos.vector_to(self.pos);
        let len2 = dir.magnitude2();

        Lixel {
            color,
            dir: dir.normalize(),
            len2,
        }
    }
}
