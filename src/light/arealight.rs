use cgmath::InnerSpace;

use crate::light::{Attenuation, Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, GridSamples, Maxel, Vector, Vectorx, RF};

#[derive(Debug)]
pub struct AreaLight<F: Float> {
    pub attn: Attenuation<F>,
    pos: Vector<F>,
    #[allow(dead_code)]
    dir: Vector<F>,
    #[allow(dead_code)]
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
    pub const ICON: &'static str = egui_phosphor::regular::HEADLIGHTS;

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
    sceneobject_impl_body!("Area Light", Self::ICON);
}

impl<F: Float> Interactive<F> for AreaLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use egui::Slider;

        use crate::gui::controls;

        let mut res = false;

        res |= controls::color(ui, &mut self.color, "Color");
        res |= controls::attenuation(ui, &mut self.attn);
        res |= controls::position(ui, &mut self.pos, "Position");
        res |= controls::position(ui, &mut self.dir, "Direction");
        res |= controls::position(ui, &mut self.upd, "Up direction");

        ui.label("X resolution");
        res |= ui.add(Slider::new(&mut self.xres, 1..=32)).changed();
        ui.end_row();

        ui.label("Y resolution");
        res |= ui.add(Slider::new(&mut self.yres, 1..=32)).changed();
        ui.end_row();

        ui.label("Width");
        res |= ui
            .add(Slider::new(&mut self.width, F::ZERO..=F::from_u32(10)).clamp_to_range(false))
            .changed();
        ui.end_row();

        ui.label("Height");
        res |= ui
            .add(Slider::new(&mut self.height, F::ZERO..=F::from_u32(10)).clamp_to_range(false))
            .changed();
        ui.end_row();

        if res {
            let (dir1, dir2) = Self::compute_dirs(self.dir, self.upd);
            self.dir1 = dir1;
            self.dir2 = dir2;
        }
        res
    }
}

impl<F: Float> Light<F> for AreaLight<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let mut color = Color::BLACK;

        /* let mut rng = rand::thread_rng(); */

        let (xres, yres) = if maxel.flags.contains(RF::Preview) {
            (1, 1)
        } else {
            (
                (self.xres >> maxel.lvl).max(1),
                (self.yres >> maxel.lvl).max(1),
            )
        };

        let dist = GridSamples::new(self.width, self.height, xres, yres);

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

            color += rt.ray_shadow(maxel, &lixel).unwrap_or(lixel.color);
        }

        color = color / F::from_u32(xres * yres);

        let dir = maxel.pos.vector_to(self.pos);
        let len2 = dir.magnitude2();

        Lixel {
            color,
            dir: dir.normalize(),
            len2,
        }
    }
}
