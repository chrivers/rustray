use std::sync::RwLock;

#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use image::{ColorType, ImageBuffer, Rgb};

use crate::scene::BoxScene;
use crate::tracer::Tracer;
use crate::types::{Float, RResult, TimeSlice};

mod pbar {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn init(range: u64) -> ProgressBar {
        let pb = ProgressBar::new(range);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.bright.cyan/blue}] line {pos}/{len} ({eta})").unwrap()
                .progress_chars("*>-")
        );
        pb
    }
}

fn draw_image<F: Float>(
    time: &mut TimeSlice,
    tracer: Tracer<F>,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let pb = pbar::init(height as u64);

    let mut img = ImageBuffer::new(width, height);

    time.set("render");

    let camera = &tracer.scene().cameras[0];

    #[cfg(feature = "rayon")]
    let indices = (0..height).into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let indices = (0..height).into_iter();

    let lines: Vec<_> = indices
        .progress_with(pb)
        .map(|y| tracer.generate_span(camera, y))
        .collect();

    time.set("copy");

    for (y, line) in lines.iter().enumerate() {
        for (x, pixel) in line.pixels.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, Rgb(pixel.to_array()));
        }
    }

    img
}

pub fn run<F>(scene: BoxScene<F>, width: u32, height: u32) -> RResult<()>
where
    F: Float + From<f32>,
{
    let mut time = TimeSlice::new("startup");

    let scene = RwLock::new(scene);

    let img = draw_image(&mut time, Tracer::new(scene.read().unwrap()), width, height);

    time.set("write");
    image::save_buffer(
        "output.png",
        &img,
        img.width(),
        img.height(),
        ColorType::Rgb8,
    )?;

    info!("render complete");
    time.stop();
    time.show();
    Ok(())
}
