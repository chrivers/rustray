#![feature(box_patterns)]
#![feature(const_trait_impl)]
#![feature(effects)]

#[macro_use]
extern crate log;

pub mod demoscene;
pub mod download;
pub mod format;
pub mod frontend;
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

#[cfg(feature = "fixedpoint1")]
pub mod fixedpoint;
#[cfg(feature = "fixedpoint2")]
pub mod fixedpoint2;
