use super::samp_util::*;

use std::fmt;
use std::fmt::Debug;

use perlin2d::PerlinNoise2D;

pub struct Perlin {
    w: u32,
    h: u32,
    pn: PerlinNoise2D,
}

impl Debug for Perlin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Perlin {{ w:")?;
        Debug::fmt(&self.w, f)?;
        write!(f, ", h:")?;
        Debug::fmt(&self.h, f)?;
        f.write_str("}")
    }
}

impl Perlin {
    #[must_use]
    pub fn new(w: u32, h: u32) -> Self {
        let pn = PerlinNoise2D::new(
            // `octaves`     - The amount of detail in Perlin noise.
            5,
            // `amplitude`   - The maximum absolute value that the Perlin noise can output.
            1.0,
            // `frequency`   - The number of cycles per unit length that the Perlin noise outputs.
            1.0,
            // `persistence` - A multiplier that determines how quickly the amplitudes diminish for each successive octave in a Perlin-noise function.
            0.8,
            // `lacunarity`  - A multiplier that determines how quickly the frequency increases for each successive octave in a Perlin-noise function.
            1.31,
            // `scale`       - A Tuple. A number that determines at what distance to view the noisemap.
            (1.0 / (w as f64), 1.0 / (h as f64)),
            // `bias`        - Amount of change in Perlin noise. Used, for example, to make all Perlin noise values positive.
            0.0,
            // `seed`        - A value that changes the output of a coherent-noise function.
            1,
        );
        Self { w, h, pn }
    }
}

impl<F: Float + Texel> Sampler<F, F> for Perlin {
    fn sample(&self, uv: Point<F>) -> F {
        let x = uv.x.to_f32().unwrap() as f64;
        let y = uv.y.to_f32().unwrap() as f64;
        F::from_f64(self.pn.get_noise(x, y))
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) -> bool {
        ui.label(name);
        egui::CollapsingHeader::new("Perlin")
            .default_open(true)
            .show(ui, |ui| {
                let mut res = false;
                res |= ui
                    .add(Slider::new(&mut self.w, 0..=10).text("Width"))
                    .changed();
                res |= ui
                    .add(Slider::new(&mut self.h, 0..=10).text("Height"))
                    .changed();
                res
            })
            .body_returned
            .unwrap_or(false)
    }
}
