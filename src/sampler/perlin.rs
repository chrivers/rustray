use super::samp_util::*;

use num_traits::ToPrimitive;

use perlin2d::PerlinNoise2D;

pub struct Perlin
{
    w: u32,
    h: u32,
    pn: PerlinNoise2D,
}

impl Perlin
{
    pub fn new(w: u32, h: u32) -> Self
    {
        let pn = PerlinNoise2D::new(
            5,                  // `octaves`     - The amount of detail in Perlin noise.
            1.0,                // `amplitude`   - The maximum absolute value that the Perlin noise can output.
            1.0,                // `frequency`   - The number of cycles per unit length that the Perlin noise outputs.
            0.8,                // `persistence` - A multiplier that determines how quickly the amplitudes diminish for each successive octave in a Perlin-noise function.
            1.31,               // `lacunarity`  - A multiplier that determines how quickly the frequency increases for each successive octave in a Perlin-noise function.
            (1.0 / (w as f64),
             1.0 / (h as f64)), // `scale`       - A Tuple. A number that determines at what distance to view the noisemap.
            0.0,                // `bias`        - Amount of change in Perlin noise. Used, for example, to make all Perlin noise values positive.
            1                   // `seed`        - A value that changes the output of a coherent-noise function.
        );
        Self { w, h, pn }
    }
}

impl<F: Float> Sampler<F, F> for Perlin
{
    fn sample(&self, uv: Point<F>) -> F
    {
        let x = uv.x.to_f32().unwrap() as f64;
        let y = uv.y.to_f32().unwrap() as f64;
        F::from_f32(self.pn.get_noise(x, y) as f32)
    }

    fn raw_sample(&self, uv: Point<u32>) -> F
    {
        let x = uv.x.to_f32().unwrap().trunc() as f64;
        let y = uv.y.to_f32().unwrap().trunc() as f64;
        F::from_f32(self.pn.get_noise(x, y) as f32)
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.w, self.h)
    }
}
