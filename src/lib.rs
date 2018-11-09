#![allow(unused_variables,unused_imports)]
//! Generic storage of tiles
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate byteorder;
extern crate rusqlite;
extern crate serde_json;

mod modtile;
mod tilestash;
mod zxy;
mod mbtiles;

pub use modtile::ModTileStore;
pub use tilestash::TileStashStore;
pub use zxy::ZXYStore;
pub use mbtiles::MbtilesStore;

use std::path::{Path, PathBuf};
use std::fmt::Debug;

use serde_json::Value as JSON;

type Result<T> = std::result::Result<T, failure::Error>;

/// Trait for working with tile stores
pub trait TileStoreTrait {

    fn new(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Self> where Self: std::marker::Sized;

    /// Given a path, attempt to open it with this TileStore, returning None if that path is not in
    /// this format.
    fn attempt_open(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Option<Self>> where Self: std::marker::Sized;

    /// Get this tile's bytes
    /// Err(...) if there was an error reading the file
    /// Ok(None) if there was no error reading
    fn get_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>>;

    /// Set the contents of this tile to this
    fn set_tile_zxy(&self, z: u8, x: u32, y: u32, content: &[u8]) -> Result<()>;

    /// True iff this store has this tile.
    /// Error if there was a problem checking this
    fn has_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<bool>;

    fn pathish_for_tile_zxy(&self, z: u8, x: u32, y: u32) -> Option<PathBuf>;

    fn tilejson(&self) -> Result<Option<JSON>> {
        unimplemented!()
    }
}


/// A generic Tilestore
#[derive(Debug)]
pub enum TileStore {
    ModTile(ModTileStore),
    TileStash(TileStashStore),
    ZXY(ZXYStore),
    Mbtiles(MbtilesStore),
}

impl TileStoreTrait for TileStore {
    fn new(_p: impl AsRef<Path>+Debug, _file_ext: impl Into<String>) -> Result<Self> {
        Err(format_err!("Cannot create a new generic TileStore"))
    }

    fn attempt_open(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Option<Self>> {
        // FIXME This is silly, remove it
        // Maybe use AsRef<str> for file_ext, since some tilestores don't use it
        let file_ext: String = file_ext.into();

        if let Some(mt) = ModTileStore::attempt_open(&p, file_ext.clone())? {
            debug!("Detected path {:?} as ModTileStore", p);
            Ok(Some(TileStore::ModTile(mt)))
        } else if let Some(ts) = TileStashStore::attempt_open(&p, file_ext.clone())? {
            debug!("Detected path {:?} as TileStashStore", p);
            Ok(Some(TileStore::TileStash(ts)))
        } else if let Some(zxy) = ZXYStore::attempt_open(&p, file_ext.clone())? {
            debug!("Detected path {:?} as ZXY", p);
            Ok(Some(TileStore::ZXY(zxy)))
        } else if let Some(mb) = MbtilesStore::attempt_open(&p, file_ext.clone())? {
            debug!("Detected path {:?} as mbtiles", p);
            Ok(Some(TileStore::Mbtiles(mb)))
        } else {
            Ok(None)
        }
    }

    fn get_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
        match self {
            TileStore::ModTile(m) => m.get_tile_zxy(z, x, y),
            TileStore::TileStash(ts) => ts.get_tile_zxy(z, x, y),
            TileStore::ZXY(s) => s.get_tile_zxy(z, x, y),
            TileStore::Mbtiles(s) => s.get_tile_zxy(z, x, y),
        }
    }

    fn set_tile_zxy(&self, z: u8, x: u32, y: u32, content: &[u8]) -> Result<()> {
        match self {
            TileStore::ModTile(m) => m.set_tile_zxy(z, x, y, content),
            TileStore::TileStash(ts) => ts.set_tile_zxy(z, x, y, content),
            TileStore::ZXY(s) => s.set_tile_zxy(z, x, y, content),
            TileStore::Mbtiles(s) => s.set_tile_zxy(z, x, y, content),
        }
    }

    fn has_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<bool> {
        match self {
            TileStore::ModTile(m) => m.has_tile_zxy(z, x, y),
            TileStore::TileStash(ts) => ts.has_tile_zxy(z, x, y),
            TileStore::ZXY(s) => s.has_tile_zxy(z, x, y),
            TileStore::Mbtiles(s) => s.has_tile_zxy(z, x, y),
        }
    }

    fn pathish_for_tile_zxy(&self, z: u8, x: u32, y: u32) -> Option<PathBuf> {
        match self {
            TileStore::ModTile(m) => m.pathish_for_tile_zxy(z, x, y),
            TileStore::TileStash(ts) => ts.pathish_for_tile_zxy(z, x, y),
            TileStore::ZXY(s) => s.pathish_for_tile_zxy(z, x, y),
            TileStore::Mbtiles(s) => s.pathish_for_tile_zxy(z, x, y),
        }
    }

    fn tilejson(&self) -> Result<Option<JSON>> {
        match self {
            TileStore::ModTile(s) => s.tilejson(),
            TileStore::TileStash(s) => s.tilejson(),
            TileStore::ZXY(s) => s.tilejson(),
            TileStore::Mbtiles(s) => s.tilejson(),
        }
    }
}
