#![feature(box_patterns)]

#[macro_use]
extern crate log;

pub mod types;
pub mod scene;
pub mod tracer;
pub mod material;
pub mod geometry;
pub mod sampler;
pub mod download;
pub mod format;
pub mod demoscene;

use crate::types::{Color, Point, Vector, Float};
use crate::types::vector::Vectorx;
use crate::scene::Light;
use crate::sampler::{Sampler, SamplerExt, DynSampler};
use crate::material::*;
