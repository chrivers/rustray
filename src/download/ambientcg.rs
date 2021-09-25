use std::io::Write;
use std::fs::File;
use std::path::PathBuf;

use reqwest;

use super::TextureDownloader;
use crate::lib::Error;

#[allow(non_camel_case_types)]
pub enum ACGQuality
{
    PNG_1K,
    PNG_2K,
    PNG_4K,
}

impl ToString for ACGQuality
{
    fn to_string(&self) -> String
    {
        match self
        {
            ACGQuality::PNG_1K => "1K-PNG",
            ACGQuality::PNG_2K => "2K-PNG",
            ACGQuality::PNG_4K => "4K-PNG",
        }.to_owned()
    }
}

pub struct ACGDownloader
{
    root: PathBuf,
    qual: ACGQuality,
}

impl ACGDownloader
{
    pub fn new(root: PathBuf, qual: ACGQuality) -> Self
    {
        Self { root: root, qual }
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
        format!("{}_{}.zip", name, self.qual.to_string())
    }

    fn cached(&self, name: &str) -> bool {
        self.fullname(name).exists()
    }

    fn download(&self, name: &str) -> Result<PathBuf, Error>
    {
        let zipname = self.filename(name);
        let path = self.fullname(name);

        if path.exists() {
            return Ok(path)
        }
        let target = format!("https://ambientcg.com/get?file={}", zipname);
        let response = reqwest::blocking::get(target)?;

        let mut file = File::create(&path)?;
        let content = response.bytes()?;
        file.write_all(&content)?;
        Ok(path)
    }
}
