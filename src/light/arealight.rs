use cgmath::InnerSpace;

/* use rand::Rng; */

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::maxel::Maxel;
use crate::types::{Color, Float, Vector};
use crate::Vectorx;

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

use super::Attenuation;

#[derive(Debug)]
pub struct AreaLight<F: Float> {
    pub attn: Attenuation<F>,
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub upd: Vector<F>,
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
        let dir1 = dir.cross(upd);
        let dir2 = dir.cross(dir1);

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
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Area light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        color_ui(ui, &mut self.color, "Color");
                        ui.end_row();

                        ui.label("Falloff d^0");
                        ui.add(egui::Slider::new(&mut self.attn.a, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        ui.label("Falloff d^1");
                        ui.add(egui::Slider::new(&mut self.attn.b, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        ui.label("Falloff d^2");
                        ui.add(egui::Slider::new(&mut self.attn.c, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        position_ui(ui, &mut self.pos, "Position");
                        position_ui(ui, &mut self.dir, "Direction");
                    })
            });
    }
}

impl<F: Float> Light<F> for AreaLight<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let mut color = Color::BLACK;

        /* let mut rng = rand::thread_rng(); */

        for ys in 0..self.yres {
            /* let ry: F = (rng.gen::<F>() - F::HALF) * self.height; */
            let ry = ((F::from_u32(ys) / F::from_u32(self.yres)) - F::HALF) * self.height;
            for xs in 0..self.xres {
                /* let rx: F = (rng.gen::<F>() - F::HALF) * self.width; */
                let rx = ((F::from_u32(xs) / F::from_u32(self.xres)) - F::HALF) * self.width;

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
