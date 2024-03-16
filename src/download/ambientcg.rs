use std::fmt::Display;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

use super::TextureDownloader;
use crate::types::RResult;

#[allow(non_camel_case_types)]
pub enum ACGQuality
{
    PNG_1K,
    PNG_2K,
    PNG_4K,
}

impl Display for ACGQuality
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self
        {
            ACGQuality::PNG_1K => write!(f, "1K-PNG"),
            ACGQuality::PNG_2K => write!(f, "2K-PNG"),
            ACGQuality::PNG_4K => write!(f, "4K-PNG"),
        }
    }
}

pub struct ACGDownloader
{
    root: PathBuf,
    qual: ACGQuality,
}

impl ACGDownloader
{
    pub fn new(root: PathBuf, qual: ACGQuality) -> RResult<Self>
    {
        std::fs::create_dir_all(&root)?;
        Ok(Self { root, qual })
    }

    pub fn fullname(&self, name: &str) -> PathBuf
    {
        let mut pb = self.root.clone();
        pb.push(self.filename(name));
        pb
    }
}

impl TextureDownloader for ACGDownloader
{
    fn filename(&self, name: &str) -> String
    {
        format!("{}_{}.zip", name, self.qual)
    }

    fn cached(&self, name: &str) -> bool {
        self.fullname(name).exists()
    }

    fn download(&self, name: &str) -> RResult<PathBuf>
    {
        let path = self.fullname(name);
        if path.exists() {
            return Ok(path)
        }

        let target = format!("https://ambientcg.com/get?file={}", self.filename(name));
        let response = reqwest::blocking::get(target)?;

        let mut file = File::create(&path)?;
        let content = response.bytes()?;
        file.write_all(&content)?;
        Ok(path)
    }
}
