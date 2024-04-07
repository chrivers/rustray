use crate::types::Float;

pub struct GridSamples<F: Float> {
    width: F,
    height: F,
    xres: u32,
    yres: u32,
}

pub struct GridSamplesIter<'a, F: Float> {
    samples: &'a GridSamples<F>,
    count: u32,
    total: u32,
}

impl<'a, F: Float> GridSamples<F> {
    pub const fn new(width: F, height: F, xres: u32, yres: u32) -> Self {
        Self {
            width,
            height,
            xres,
            yres,
        }
    }

    pub const fn iter(&'a self) -> GridSamplesIter<'a, F> {
        GridSamplesIter {
            samples: self,
            count: 0,
            total: self.xres * self.yres,
        }
    }
}

impl<'a, F: Float> Iterator for GridSamplesIter<'a, F> {
    type Item = (F, F);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.total {
            return None;
        }

        self.count += 1;

        let samples = self.samples;

        let y = self.count / samples.xres;
        let x = self.count % samples.xres;

        let ry = ((F::from_u32(y) / F::from_u32(samples.yres)) - F::HALF) * samples.height;
        let rx = ((F::from_u32(x) / F::from_u32(samples.xres)) - F::HALF) * samples.width;

        Some((rx, ry))
    }
}
