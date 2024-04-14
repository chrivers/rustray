#![feature(box_patterns)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![warn(
    clippy::all,
    clippy::correctness,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::perf,
    clippy::style
)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::inline_always,
    clippy::many_single_char_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::similar_names
)]

#[macro_use]
extern crate log;

pub mod debug;
pub mod demoscene;
pub mod download;
pub mod engine;
pub mod format;
pub mod frontend;
pub mod geometry;
pub mod light;
pub mod material;
pub mod sampler;
pub mod scene;
pub mod tracer;
pub mod types;

#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "fixedpoint1")]
pub mod fixedpoint;
#[cfg(feature = "fixedpoint2")]
pub mod fixedpoint2;
