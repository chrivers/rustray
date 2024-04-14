#![feature(box_patterns)]
#![feature(const_trait_impl)]
#![feature(effects)]
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
    clippy::items_after_statements,
    clippy::many_single_char_names,
    clippy::map_unwrap_or,
    clippy::mismatching_type_param_order,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::significant_drop_tightening,
    clippy::similar_names,
    /* clippy::fallible_impl_from, */
    /* clippy::significant_drop_in_scrutinee, */
    /* clippy::must_use_candidate, */
    /* clippy::unreadable_literal, */
    /* clippy::use_self, */
    /* clippy::missing_const_for_fn, */
    /* clippy::type_repetition_in_bounds, */
    /* clippy::return_self_not_must_use, */
    /* clippy::trait_duplication_in_bounds, */
    /* clippy::inconsistent_struct_constructor, */
    /* clippy::match_same_arms, */
    /* clippy::unused_self */
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
