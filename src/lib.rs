#![feature(box_patterns)]

#[macro_use]
extern crate log;

pub mod demoscene;
pub mod download;
pub mod format;
pub mod geometry;
pub mod material;
pub mod sampler;
pub mod scene;
pub mod tracer;
pub mod types;

use crate::material::*;
use crate::sampler::{DynSampler, Sampler, SamplerExt};
use crate::scene::Light;
use crate::types::vector::Vectorx;
use crate::types::{Color, Float, Point, Vector};
