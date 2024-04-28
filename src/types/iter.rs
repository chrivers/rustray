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
    xoffset: F,
    yoffset: F,
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
            xoffset: (self.width / F::from_u32(self.xres)) / F::TWO,
            yoffset: (self.height / F::from_u32(self.yres)) / F::TWO,
        }
    }
}

impl<'a, F: Float> Iterator for GridSamplesIter<'a, F> {
    type Item = (F, F);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.total {
            return None;
        }

        let samples = self.samples;

        let y = F::from_u32(self.count / samples.xres);
        let x = F::from_u32(self.count % samples.xres);

        self.count += 1;

        let rx = (x / F::from_u32(samples.xres)) * samples.width + self.xoffset;
        let ry = (y / F::from_u32(samples.yres)) * samples.height + self.yoffset;

        Some((rx, ry))
    }
}

#[cfg(test)]
mod tests {
    use super::GridSamples;

    #[test]
    fn test_grid_samples_single_point_centered() {
        let g = GridSamples::new(20.0, 20.0, 1, 1);
        assert_eq!(g.iter().collect::<Vec<_>>().as_slice(), [(10.0, 10.0)]);
    }

    #[test]
    fn test_grid_samples_x_centered() {
        let g = GridSamples::new(8.0, 0.0, 4, 1);
        assert_eq!(
            g.iter().collect::<Vec<_>>().as_slice(),
            [(1.0, 0.0), (3.0, 0.0), (5.0, 0.0), (7.0, 0.0)]
        );
    }

    #[test]
    fn test_grid_samples_y_centered() {
        let g = GridSamples::new(0.0, 8.0, 1, 4);
        assert_eq!(
            g.iter().collect::<Vec<_>>().as_slice(),
            [(0.0, 1.0), (0.0, 3.0), (0.0, 5.0), (0.0, 7.0)]
        );
    }

    #[test]
    fn test_grid_samples_uniform() {
        let g = GridSamples::new(8.0, 8.0, 2, 2);
        assert_eq!(
            g.iter().collect::<Vec<_>>().as_slice(),
            [(2.0, 2.0), (6.0, 2.0), (2.0, 6.0), (6.0, 6.0)]
        );
    }
}
