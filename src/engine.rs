use std::sync::{Arc, RwLock};

use workerpool::thunk::{Thunk, ThunkWorker};
use workerpool::Pool;

use crate::scene::BoxScene;
use crate::tracer::Tracer;
use crate::{Color, Float};

pub struct RenderSpan<F: Float> {
    pub line: u32,
    pub pixels: Vec<Color<F>>,
}

pub struct RenderEngine<F: Float> {
    lock: Arc<RwLock<BoxScene<F>>>,
    pool: Pool<ThunkWorker<RenderSpan<F>>>,
    pub rx: crossbeam_channel::Receiver<RenderSpan<F>>,
    tx: crossbeam_channel::Sender<RenderSpan<F>>,
    #[allow(dead_code)]
    width: u32,
    height: u32,
}

impl<F: Float> RenderEngine<F> {
    pub fn new(lock: Arc<RwLock<BoxScene<F>>>, width: u32, height: u32) -> Self {
        let (tx, rx) = crossbeam_channel::bounded::<RenderSpan<F>>(2000);

        let pool = Pool::<ThunkWorker<RenderSpan<F>>>::new(32);

        Self {
            lock,
            pool,
            rx,
            tx,
            width,
            height,
        }
    }

    pub fn render_lines(&self, a: u32, b: u32) {
        for x in a..b {
            let lock = self.lock.clone();
            self.pool.execute_to(
                self.tx.clone(),
                Thunk::of(move || {
                    let scene = lock.read().unwrap();
                    let tracer = Tracer::new(scene);
                    let camera = &tracer.scene().cameras[0];

                    tracer.generate_span(camera, x)
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
