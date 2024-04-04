use std::sync::{Arc, RwLock};

use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

use crate::scene::BoxScene;
use crate::tracer::Tracer;
use crate::{Color, Float};

pub struct RenderSpan<F: Float> {
    pub line: u32,
    pub mult_x: u32,
    pub mult_y: u32,
    pub pixels: Vec<Color<F>>,
}

pub struct RenderEngine<F: Float> {
    pool: Pool<ThunkWorker<RenderSpan<F>>>,
    pub rx: crossbeam_channel::Receiver<RenderSpan<F>>,
    tx: crossbeam_channel::Sender<RenderSpan<F>>,
    #[allow(dead_code)]
    width: u32,
    height: u32,
}

impl<F: Float> RenderEngine<F> {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let (tx, rx) = crossbeam_channel::bounded::<RenderSpan<F>>(2000);

        let pool = Pool::<ThunkWorker<RenderSpan<F>>>::new(32);

        Self {
            pool,
            rx,
            tx,
            width,
            height,
        }
    }

    pub fn render_lines(&self, lock: Arc<RwLock<BoxScene<F>>>, a: u32, b: u32) {
        self.render_lines_by_step(lock, a, b, 1, 1);
    }

    pub fn render_lines_by_step(
        &self,
        lock: Arc<RwLock<BoxScene<F>>>,
        a: u32,
        b: u32,
        step_x: u32,
        step_y: u32,
    ) {
        for y in (a..b).step_by(step_y as usize) {
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

    pub fn render_normals(&self, lock: Arc<RwLock<BoxScene<F>>>, a: u32, b: u32) {
        for x in a..b {
            let lock = lock.clone();
            self.pool.execute_to(
                self.tx.clone(),
                Thunk::of(move || {
                    let scene = lock.read().unwrap();
                    let tracer = Tracer::new(scene);
                    let camera = tracer.scene().cameras[0];

                    tracer.generate_normal_span(&camera, x)
                }),
            );
        }
    }

    #[must_use]
    pub fn progress(&self) -> f32 {
        let act = self.pool.queued_count() as f32;
        let max = self.height as f32;
        1.0 - (act / max)
    }
}
