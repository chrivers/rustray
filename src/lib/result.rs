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
    ZipError(#[from] zip::result::ZipError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("parse error")]
    ParseError()
}
