use thiserror::Error;

use crate::{engine::RenderSpan, Float};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    PestError2(#[from] Box<pest::error::Error<crate::format::sbt2::Rule>>),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    #[error(transparent)]
    ObjError(#[from] obj::ObjError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    MtlLibsLoadError(#[from] obj::MtlLibsLoadError),

    #[error("parse error")]
    ParseError(&'static str),

    #[error("missing attribute {0:?}")]
    ParseMissingKey(String),

    #[error("unsupported parse element {0:?}")]
    ParseUnsupported(String),

    #[error(transparent)]
    BuildError(#[from] rtbvh::BuildError),

    #[error("Crossbeam send error")]
    CrossbeamSend,

    #[cfg(feature = "gui")]
    #[error(transparent)]
    EFrame(#[from] eframe::Error),
}

pub type RResult<F> = Result<F, Error>;

impl From<pest::error::Error<crate::format::sbt2::Rule>> for Error {
    fn from(value: pest::error::Error<crate::format::sbt2::Rule>) -> Self {
        Self::PestError2(Box::new(value))
    }
}

impl<F: Float> From<crossbeam_channel::SendError<RenderSpan<F>>> for Error {
    fn from(_value: crossbeam_channel::SendError<RenderSpan<F>>) -> Self {
        Self::CrossbeamSend
    }
}
