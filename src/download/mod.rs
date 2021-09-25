use super::Error;
use std::path::PathBuf;

pub trait TextureDownloader
{
    fn filename(&self, name: &str) -> String;
    fn download(&self, name: &str) -> Result<PathBuf, Error>;
    fn cached(&self, name: &str) -> bool;
}

pub mod ambientcg;
pub use ambientcg::{ACGDownloader, ACGQuality};
