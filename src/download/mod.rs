use crate::types::RResult;
use std::path::PathBuf;

pub trait TextureDownloader {
    fn filename(&self, name: &str) -> String;
    fn download(&self, name: &str) -> RResult<PathBuf>;
    fn cached(&self, name: &str) -> bool;
}

pub mod ambientcg;
pub use ambientcg::{ACGDownloader, ACGQuality};
