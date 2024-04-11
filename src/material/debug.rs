use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub enum DebugType {
    ColorPos,
    ColorNormal,
    ColorST,
    ColorUV,
}

#[derive(Copy, Clone, Debug)]
pub struct ColorDebug<F: Float> {
    dt: DebugType,
    scale: F,
}

impl<F: Float> ColorDebug<F> {
    pub const fn new(dt: DebugType, scale: F) -> Self {
        Self { dt, scale }
    }

    pub const fn pos() -> Self {
        Self::new(DebugType::ColorPos, F::ONE)
    }

    pub const fn normal() -> Self {
        Self::new(DebugType::ColorNormal, F::ONE)
    }

    pub const fn st() -> Self {
        Self::new(DebugType::ColorST, F::ONE)
    }

    pub const fn uv() -> Self {
        Self::new(DebugType::ColorUV, F::ONE)
    }
}

impl<F: Float> Material<F> for ColorDebug<F> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        let res = match self.dt {
            DebugType::ColorPos => {
                let mut n = maxel.pos / F::from_f32(32.0);
                n.x += F::ONE;
                n.y += F::ONE;
                Color::new(n.x, n.y, n.z)
            }
            DebugType::ColorNormal => {
                let n = maxel.nml();
                Color::new(n.x.abs(), n.y.abs(), n.z.abs())
            }
            DebugType::ColorST => {
                let uv = maxel.uv();
                Color::new(uv.x, F::ZERO, uv.y)
            }
            DebugType::ColorUV => {
                let st = maxel.st();
                let w = F::ONE - st.x - st.y;
                Color::new(st.x, w, st.y)
            }
        };
        res * self.scale
    }
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for ColorDebug<F> {}

impl<F: Float> SceneObject<F> for ColorDebug<F> {
    sceneobject_impl_body!("Color Debug");
}
