use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error
{
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    PestError(#[from] pest::error::Error<crate::format::sbt::Rule>),

    #[error(transparent)]
    PestError2(#[from] pest::error::Error<crate::format::sbt2::Rule>),

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

    #[error("parse error")]
    ParseError(&'static str),

    #[error("missing attribute {0:?}")]
    ParseMissingKey(String),

    #[error("unsupported parse element {0:?}")]
    ParseUnsupported(String)
}

pub type RResult<F> = Result<F, Error>;
