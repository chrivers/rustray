use std::path::PathBuf;

use crate::types::RResult;

mod ambientcg;
pub use ambientcg::{ACGDownloader, ACGQuality};

pub trait TextureDownloader {
    fn filename(&self, name: &str) -> String;
    fn download(&self, name: &str) -> RResult<PathBuf>;
    fn cached(&self, name: &str) -> bool;
}
