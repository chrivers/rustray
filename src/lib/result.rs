use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error
{
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
