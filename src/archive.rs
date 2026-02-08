use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, Read};

use tar::Archive;
use xz2::read::XzDecoder;


/// TODO: continue and test for plugins and add packer

pub struct Archives {}

impl Archives {

    pub fn open(path: PathBuf) -> crate::Result<Archive<XzDecoder<BufReader<File>>>> {
        let file = File::open(path)?;
        let tar = XzDecoder::new(BufReader::new(file));
        Ok(Archive::new(tar))
    }

    pub fn exists(path: PathBuf, find: &str) -> crate::Result<bool> {
        let mut archive  = Archives::open(path)?;
        Ok(match archive.entries()?.find(|p| p.as_ref().unwrap().path().unwrap().ends_with(find)) {
            Some(_) => true,
            None => false
        })
    }

    pub fn find_file(path: PathBuf, find: &str) -> crate::Result<String> {
        let mut archive  = Archives::open(path)?;
        let mut s = String::new();
        let _ = match archive.entries()?.find(|p| p.as_ref().unwrap().path().unwrap().ends_with(find)) {
            Some(mut data) => Ok(data.as_mut().unwrap().read_to_string(&mut s)),
            None => Err(crate::Error::Unknown(format!("file {} not found", find)))
        }?;
        Ok(s)
    }

}