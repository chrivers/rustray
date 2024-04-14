use std::sync::{Arc, RwLock};

#[cfg(feature = "gui")]
use egui::ColorImage;
use image::{ImageBuffer, Rgba};
use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

use crate::scene::{BoxScene, RayTracer};
use crate::tracer::Tracer;
use crate::types::{Color, Float};

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

    pub fn render_all(&mut self, lock: &Arc<RwLock<BoxScene<F>>>) {
        self.render_lines(lock, 0, self.img.height());
    }

    pub fn render_lines(&mut self, lock: &Arc<RwLock<BoxScene<F>>>, a: u32, b: u32) {
        let color = Rgba(Color::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4());

        for y in 0..self.img.height() {
            self.img.put_pixel(0, y, color);
        }

        self.render_lines_by_step(lock, a, b, 1, 1);
    }

    pub fn render_lines_by_step(
        &mut self,
        lock: &Arc<RwLock<BoxScene<F>>>,
        a: u32,
        b: u32,
        step_x: u32,
        step_y: u32,
    ) {
        for y in (a..b).step_by(step_y as usize) {
            let dirty = &mut self.dirty[y as usize];
            if *dirty {
                continue;
            }
            *dirty = true;

            let lock = lock.clone();
            self.pool.execute_to(
                self.tx.clone(),
                Thunk::of(move || {
                    let scene = lock.read().unwrap();
                    let tracer = Tracer::new(scene);
                    let camera = &tracer.scene().cameras[0];

                    let mut span = tracer.generate_span_coarse(camera, y + step_y / 2, step_x);
                    span.mult_y = step_y;
                    span.line -= step_y / 2;
                    span
                }),
            );
        }
    }

    pub fn render_all_by_step(
        &mut self,
        lock: &Arc<RwLock<BoxScene<F>>>,
        step_x: u32,
        step_y: u32,
    ) {
        self.render_lines_by_step(lock, 0, self.img.height(), step_x, step_y);
    }

    #[must_use]
    #[cfg(feature = "gui")]
    pub fn get_epaint_image(&self) -> ColorImage {
        let size = [self.img.width() as usize, self.img.height() as usize];

        ColorImage::from_rgba_unmultiplied(size, self.img.as_flat_samples().as_slice())
    }

    pub fn render_normals(&mut self, lock: &Arc<RwLock<BoxScene<F>>>) {
        let color = Rgba(Color::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4());
        for y in 0..self.img.height() {
            self.img.put_pixel(0, y, color);
        }

        for y in 0..self.img.height() {
            let lock = lock.clone();
            self.pool.execute_to(
                self.tx.clone(),
                Thunk::of(move || {
                    let scene = lock.read().unwrap();
                    let tracer = Tracer::new(scene);
                    let camera = tracer.scene().cameras[0];

                    tracer.generate_normal_span(&camera, y)
                }),
            );
        }
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
