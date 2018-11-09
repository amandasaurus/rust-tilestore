use ::{TileStoreTrait, Result, JSON};
use std::path::{PathBuf, Path};

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::Debug;
use rusqlite::{Connection, types::ToSql};

#[derive(Debug)]
pub struct MbtilesStore {
    connection: rusqlite::Connection,
}

impl TileStoreTrait for MbtilesStore {

    fn new(p: impl AsRef<Path>+Debug, _: impl Into<String>) -> Result<Self> {
        let p = p.as_ref();
        if p.exists() {
            Err(format_err!("File {:?} already exists", p))
        } else if ! p.extension().map(|s| s == "mbtiles").unwrap_or(false) {
            Err(format_err!("Filename must end with .mbtiles: {:?}", p))
        } else {
            let conn = Connection::open(p)?;
            let creation_schema = include_str!("mbtiles-schema.sql");
            conn.execute_batch(creation_schema)?;

            Ok(MbtilesStore{ connection: conn })
        }
    }

    fn attempt_open(p: impl AsRef<Path>+Debug, _: impl Into<String>) -> Result<Option<Self>> {
        let p = p.as_ref();
        if ! p.is_file() || ! p.extension().map(|s| s == "mbtiles").unwrap_or(false) {
            Ok(None)
        } else {
            Ok(Some(MbtilesStore{ connection: Connection::open(p)? }))
        }
    }

    fn get_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
        let row: u32 = 2u32.pow(z as u32) - y - 1;
        let query_res = self.connection.query_row(
            "SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ? LIMIT 1;",
            &[z as u32, x, row],
            |row| row.get(0)
        );
        match query_res {
            Ok(bytes) => Ok(Some(bytes)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set_tile_zxy(&self, z: u8, x: u32, y: u32, contents: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn has_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<bool> {
        unimplemented!()
    }

    fn pathish_for_tile_zxy(&self, z: u8, x: u32, y: u32) -> Option<PathBuf> {
        unimplemented!()
    }

    fn tilejson(&self) -> Result<Option<JSON>> {
        // silly hack, cause query_row doesn't like &[]
        let empty: Vec<u8> = vec![];
        let query_res = self.connection.query_row(
            "SELECT value FROM metadata WHERE name = 'json'",
            &empty, |row| row.get(0)
        );
        let raw_json: String = match query_res {
            Ok(bytes) => bytes,
            Err(rusqlite::Error::QueryReturnedNoRows) => { return Ok(None) },
            Err(e) => { return Err(e.into()) },
        };

        serde_json::from_str(&raw_json).map(|ok| Some(ok)).map_err(|e| format_err!("Unable to parse json {:?}", e))

    }
}
