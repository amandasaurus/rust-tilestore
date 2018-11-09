use ::{TileStoreTrait, Result};
use std::path::{PathBuf, Path};

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::Debug;

#[derive(Debug)]
pub struct TileStashStore {
    path: PathBuf,
    file_ext: String,
}

impl TileStashStore {

}

impl TileStoreTrait for TileStashStore {

    fn new(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Self> {
        let path = p.as_ref().to_path_buf();
        let file_ext: String = file_ext.into();

        // Create the directory in case it doesn't exist
        fs::create_dir_all(&path)?;

        Ok(TileStashStore{ path, file_ext })
    }

    fn attempt_open(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Option<Self>> {
        let file_ext: String = file_ext.into();
        if p.as_ref().join("0").join("000").join("000").join("000").join(format!("000.{}", file_ext)).exists() {
            TileStashStore::new(p, file_ext).map(|ts| Some(ts))
        } else {
            Ok(None)
        }
    }

    fn get_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
        trace!("TileStash {:?} query for tile {}/{}/{}", self, z, x, y);
        let tile_path = self.pathish_for_tile_zxy(z, x, y).unwrap();
        trace!("tile_path {:?}", tile_path);
        if !tile_path.exists() {
            return Ok(None);
        }
        
        let mut file = File::open(tile_path)?;
        let mut content = Vec::new();

        file.read_to_end(&mut content)?;

        Ok(Some(content))
    }

    fn set_tile_zxy(&self, z: u8, x: u32, y: u32, contents: &[u8]) -> Result<()> {
        let tile_path = self.pathish_for_tile_zxy(z, x, y).unwrap();
        let mut file = File::open(tile_path)?;
        file.write_all(contents)?;
        Ok(())
    }

    fn has_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<bool> {
        Ok(self.pathish_for_tile_zxy(z, x, y).unwrap().exists())
    }

    fn pathish_for_tile_zxy(&self, z: u8, x: u32, y: u32) -> Option<PathBuf> {
        let ts = xy_to_ts(x, y);
        let mut tile_path = self.path.clone();
        tile_path.push(format!("{}", z));
        tile_path.push(&ts[0]);
        tile_path.push(&ts[1]);
        tile_path.push(&ts[2]);
        tile_path.push(format!("{}.{}", ts[3], self.file_ext));

        Some(tile_path)
    }
}

/// Convert x & y to a TileStash (ts) safe directory parts
fn xy_to_ts(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:03}", x/1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y/1_000),
        format!("{:03}", y % 1_000),
    ]
}
