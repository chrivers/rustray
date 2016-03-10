#![allow(dead_code)]

use traits::Float;
use scene::*;
use vector::Vector;
use color::Color;
use light::Light;
use ray::Ray;
use plane;

#[derive(Debug)]
pub struct ChessPlane<F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    color: Color<F>,
}

impl<F: Float> RayTarget<F> for ChessPlane<F>
{
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>
    {
        let xs = F::from_float(4.0);
        let ys = F::from_float(4.0);

	let s;
	let t;

	if self.dir1.x != F::zero() {
	    s = hit.x/self.dir1.x;
	    if self.dir2.y != F::zero() {
		t = hit.y/self.dir2.y;
	    } else {
		t = hit.z/self.dir2.z;
	    }
	} else {
	    s = hit.y/self.dir1.y;
	    if self.dir2.x != F::zero() {
		t = hit.x/self.dir2.x;
	    } else {
		t = hit.z/self.dir2.z;
	    }
	}
	let xv = s/xs;
	let yv = t/ys;
	
        let x = (xv - xv.floor()) > F::from_float(0.5);
        let y = (yv - yv.floor()) > F::from_float(0.5);

        let self_color = if x^y {
	    Color::black()
	} else {
	    Color::white()
	};

        let m = hit.vector_to(light.pos);
        let normal = self.dir2.crossed(self.dir1);
        let light_color = light.color * self_color;
        // // let reflection_coeff = F::max(normal.cos_angle(m), (normal * (-F::one())).cos_angle(m));
        let reflection_coeff = normal.cos_angle(m);
        light_color * reflection_coeff
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        plane::ray_hit_plane(&self.pos, &self.dir1, &self.dir2, ray)
    }

}

impl<F: Float> ChessPlane<F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, color: Color<F>) -> ChessPlane<F>
    {
        ChessPlane { pos: pos, dir1: dir1, dir2: dir2, color: color }
    }
}
