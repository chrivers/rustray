use std::sync::Arc;

use parking_lot::RwLock;

#[cfg(feature = "gui")]
use egui::ColorImage;
use image::{ImageBuffer, Rgba};
use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

use crate::material::{ColorDebug, Material};
use crate::point;
use crate::scene::{BoxScene, RayTracer};
use crate::tracer::Tracer;
use crate::types::{Color, Float, Point, Ray};

type RenderFunc<F> = fn(&Tracer<F>, Ray<F>) -> Color<F>;

pub struct RenderJob<F: Float> {
    first_line: Option<u32>,
    last_line: Option<u32>,
    mult_x: u32,
    mult_y: u32,
    func: RenderFunc<F>,
}

impl<F: Float> RenderJob<F> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            first_line: None,
            last_line: None,
            mult_x: 1,
            mult_y: 1,
            func: |tracer, ray| {
                tracer
                    .ray_trace(&ray)
                    .map_or_else(|| tracer.scene().background, Color::clamped)
            },
        }
    }

    #[must_use]
    pub const fn with_mult_x(self, mult_x: u32) -> Self {
        Self { mult_x, ..self }
    }

    #[must_use]
    pub const fn with_mult_y(self, mult_y: u32) -> Self {
        Self { mult_y, ..self }
    }

    #[must_use]
    pub const fn with_mult(self, mult: u32) -> Self {
        Self {
            mult_x: mult,
            mult_y: mult,
            ..self
        }
    }

    #[must_use]
    pub const fn with_first_line(self, line: u32) -> Self {
        Self {
            first_line: Some(line),
            ..self
        }
    }

    #[must_use]
    pub const fn with_last_line(self, line: u32) -> Self {
        Self {
            last_line: Some(line),
            ..self
        }
    }

    #[must_use]
    pub const fn with_func(self, func: RenderFunc<F>) -> Self {
        Self { func, ..self }
    }

    #[must_use]
    pub const fn with_func_debug_normals(self) -> Self {
        Self {
            func: |tracer, ray| {
                tracer
                    .scene()
                    .intersect(&ray)
                    .map_or(Color::BLACK, |mut maxel| {
                        ColorDebug::normal().render(&mut maxel, tracer)
                    })
            },
            ..self
        }
    }

    #[must_use]
    pub fn get_lines(&self, height: u32) -> (u32, u32) {
        (
            self.first_line.unwrap_or(0),
            self.last_line.unwrap_or(height),
        )
    }

    #[must_use]
    pub const fn get_mult(&self) -> (u32, u32) {
        (self.mult_x, self.mult_y)
    }

    #[must_use]
    pub const fn get_func(&self) -> RenderFunc<F> {
        self.func
    }
}

pub struct RenderSpan<F: Float> {
    pub line: u32,
    pub mult_x: u32,
    pub mult_y: u32,
    pub pixels: Vec<Color<F>>,
}

pub struct RenderEngine<F: Float> {
    pool: Pool<ThunkWorker<RenderSpan<F>>>,
    pub img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub rx: crossbeam_channel::Receiver<RenderSpan<F>>,
    tx: crossbeam_channel::Sender<RenderSpan<F>>,
    dirty: Vec<bool>,
    #[allow(dead_code)]
    width: u32,
    height: u32,
}

pub struct RenderEngineIter<'a, F: Float> {
    engine: &'a mut RenderEngine<F>,
}

impl<'a, F: Float> Iterator for RenderEngineIter<'a, F> {
    type Item = RenderSpan<F>;

    fn next(&mut self) -> Option<Self::Item> {
        let span = self.engine.rx.try_recv().ok()?;
        for x in 0..span.mult_y {
            self.engine.dirty[(span.line + x) as usize] = false;
        }
        Some(span)
    }
}

impl<F: Float> RenderEngine<F> {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let (tx, rx) = crossbeam_channel::bounded::<RenderSpan<F>>(2000);

        let pool = Pool::<ThunkWorker<RenderSpan<F>>>::new(32);

        Self {
            pool,
            img: ImageBuffer::new(width, height),
            rx,
            tx,
            dirty: vec![false; height as usize],
            width,
            height,
        }
    }

    pub fn iter(&mut self) -> RenderEngineIter<F> {
        RenderEngineIter { engine: self }
    }

    pub fn submit(&mut self, job: &RenderJob<F>, lock: &Arc<RwLock<BoxScene<F>>>) {
        let (a, b) = job.get_lines(self.img.height());
        let (mult_x, mult_y) = job.get_mult();
        let func = job.get_func();

        let (width, height) = (self.width, self.height);
        let offset = point!(F::from_u32(mult_x) / F::TWO, F::from_u32(mult_y) / F::TWO);
        let size: Point<F> = (width, height).into();

        for y in (a..b).step_by(mult_y as usize) {
            let dirty = &mut self.dirty[y as usize];
            if *dirty {
                continue;
            }
            *dirty = true;

            let lock = Arc::clone(lock);

            self.pool.execute_to(
                self.tx.clone(),
                #[allow(clippy::significant_drop_tightening)]
                Thunk::of(move || {
                    let scene = lock.read();
                    let tracer = Tracer::new(&scene);
                    let camera = &tracer.scene().cameras[0];

                    let pixels = (0..width)
                        .step_by(mult_x as usize)
                        .map(|x| {
                            let point: Point<F> = (x, y).into();
                            let ray = camera.get_ray((point + offset) / size);
                            func(&tracer, ray)
                        })
                        .collect();

                    RenderSpan {
                        line: y,
                        mult_x,
                        mult_y,
                        pixels,
                    }
                }),
            );
        }
    }

    pub fn mark_dirty(&mut self, a: u32, b: u32) {
        let color = Rgba(Color::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4());

        for y in a..b {
            self.img.put_pixel(0, y, color);
        }
    }

    #[must_use]
    #[cfg(feature = "gui")]
    pub fn get_epaint_image(&self) -> ColorImage {
        let size = [self.img.width() as usize, self.img.height() as usize];

        ColorImage::from_rgba_unmultiplied(size, self.img.as_flat_samples().as_slice())
    }

    #[must_use]
    pub fn progress(&self) -> (usize, usize) {
        let act = self.pool.queued_count();
        let max = self.height as usize;
        (act, max)
    }

    pub fn update(&mut self) -> bool {
        let mut recv = false;

        while let Ok(span) = self.rx.try_recv() {
            for x in 0..span.mult_y {
                self.dirty[(span.line + x) as usize] = false;
            }

            for (base_x, pixel) in span.pixels.iter().enumerate() {
                let rgba = Rgba(pixel.to_array4());
                for y in 0..span.mult_y {
                    for x in 0..span.mult_x {
                        self.img
                            .put_pixel((base_x as u32) * span.mult_x + x, span.line + y, rgba);
                    }
                }
            }
            recv = true;
        }
        recv
    }
}
