use crate::scene::*;
use crate::types::{Color, Float, Vector};

#[derive(Debug)]
pub struct PointLight<F: Float> {
    pub a: F,
    pub b: F,
    pub c: F,
    pub pos: Vector<F>,
    pub color: Color<F>,
}

#[derive(Debug)]
pub struct DirectionalLight<F: Float> {
    pub dir: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> Light<F> for PointLight<F> {
    fn get_color(&self) -> Color<F> {
        self.color
    }

    fn attenuate(&self, color: Color<F>, d: F) -> Color<F> {
        color / (F::ONE + self.a + (self.b * d) + (self.c * d * d))
    }
}

impl<F: Float> Light<F> for DirectionalLight<F> {
    fn get_color(&self) -> Color<F> {
        self.color
    }

    fn attenuate(&self, color: Color<F>, _: F) -> Color<F> {
        color
    }
}

impl<F: Float> HasPosition<F> for DirectionalLight<F> {
    fn get_position(&self) -> Vector<F> {
        self.dir * F::from_f32(-100000.0)
    }
    fn set_position(&mut self, _: Vector<F>) {}
}

impl<F: Float> HasPosition<F> for PointLight<F> {
    fn get_position(&self) -> Vector<F> {
        self.pos
    }
    fn set_position(&mut self, value: Vector<F>) {
        self.pos = value;
    }
}

impl<'a, F: Float> Light<F> for Box<dyn Light<F> + 'a> {
    fn get_color(&self) -> Color<F> {
        self.as_ref().get_color()
    }

    fn attenuate(&self, color: Color<F>, d: F) -> Color<F> {
        self.as_ref().attenuate(color, d)
    }
}

impl<'a, F: Float> HasPosition<F> for Box<dyn Light<F> + 'a> {
    fn get_position(&self) -> Vector<F> {
        self.as_ref().get_position()
    }
    fn set_position(&mut self, _value: Vector<F>) {}
}
