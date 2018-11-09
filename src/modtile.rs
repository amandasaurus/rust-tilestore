use ::{TileStoreTrait, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::{PathBuf, Path};

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{SeekFrom, Seek};
use std::fmt::Debug;

#[derive(Debug)]
pub struct ModTileStore {
    path: PathBuf
}

impl TileStoreTrait for ModTileStore {

    fn new(p: impl AsRef<Path>+Debug, _dont_care: impl Into<String>) -> Result<Self> {
        let path = p.as_ref().to_path_buf();

        // Create the directory in case it doesn't exist
        fs::create_dir_all(&path)?;

        Ok(ModTileStore{ path })
    }

    fn attempt_open(p: impl AsRef<Path>+Debug, file_ext: impl Into<String>) -> Result<Option<Self>> {
        if p.as_ref().join("0").join("0").join("0").join("0").join("0").join("0.meta").exists() {
            ModTileStore::new(p, file_ext).map(|ts| Some(ts))
        } else {
            Ok(None)
        }
    }

    fn get_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
        let tile_path = self.pathish_for_tile_zxy(z, x, y).unwrap();

        if !tile_path.exists() {
            trace!("Path for {}/{}/{} is {:?}, which doesn't exist", z, x, y, tile_path);
            Ok(None)
        } else {
            trace!("Path for {}/{}/{} is {:?}, which exists", z, x, y, tile_path);
            let mut file = File::open(tile_path)?;
            
            read_tile(&mut file, z, x, y)
        }
    }

    fn set_tile_zxy(&self, _z: u8, _x: u32, _y: u32, _content: &[u8]) -> Result<()> {
        //let tile_path = self.pathish_for_tile_zxy(z, x, y).unwrap();
        unimplemented!()
    }

    fn has_tile_zxy(&self, z: u8, x: u32, y: u32) -> Result<bool> {
        // TODO should this open that file and read it?
        Ok(self.pathish_for_tile_zxy(z, x, y).unwrap().exists())
    }

    fn pathish_for_tile_zxy(&self, z: u8, x: u32, y: u32) -> Option<PathBuf> {
        // multiple of 8
        let x = x & !0b111;
        let y = y & !0b111;

        let mt = xy_to_mt(x, y);
        let mut tile_path = self.path.clone();
        tile_path.push(format!("{}", z));
        tile_path.push(&mt[0]);
        tile_path.push(&mt[1]);
        tile_path.push(&mt[2]);
        tile_path.push(&mt[3]);
        tile_path.push(format!("{}.meta", mt[4]));

        Some(tile_path)
    }

}

/// Convert x & y to a ModTile metatile directory parts
fn xy_to_mt(x: u32, y: u32) -> [String; 5] {
    // /[Z]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy]/[xxxxyyyy].meta
    // i.e. /[Z]/a/b/c/d/e.png

    let mut x = x;
    let mut y = y;

    let e = (((x & 0x0f) << 4) | (y & 0x0f)) as u8;
    x >>= 4;
    y >>= 4;

    let d = (((x & 0x0f) << 4) | (y & 0x0f)) as u8;
    x >>= 4;
    y >>= 4;

    let c = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
    x >>= 4;
    y >>= 4;

    let b = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
    x >>= 4;
    y >>= 4;

    let a = (((x & 0b000_1111 as u32) << 4) | (y & 0b000_1111 as u32)) as u8;
    //x >>= 4;
    //y >>= 4;

    [
        format!("{}", a),
        format!("{}", b),
        format!("{}", c),
        format!("{}", d),
        format!("{}", e),
    ]
}

fn read_tile<R: Read+Seek>(input: &mut R, z: u8, x: u32, y: u32) -> Result<Option<Vec<u8>>> {
    // Skip META header
    input.seek(SeekFrom::Start(4))?;
    
    let count = input.read_u32::<LittleEndian>()?;
    trace!("{}/{}/{} count is {:?}", z, x, y, count);
    let size = match count {
        0 => 0,
        1 => 1,
        4 => 2,
        16 => 4,
        64 => 8,
        _ => return Err(format_err!("Metatile has count of {}, which is unknown and unsupported", count)),
    };
    trace!("{}/{}/{} size is hence {:?}", z, x, y, size);
    let this_x = input.read_u32::<LittleEndian>()?;
    let this_y = input.read_u32::<LittleEndian>()?;
    let this_z = input.read_u32::<LittleEndian>()? as u8;

    // Check the zoom/x/y we want against what's in the tile
    if this_z != z {
        return Ok(None);
    }
    if (x & !0b111) != this_x {
        return Ok(None);
    }
    if (y & !0b111) != this_y {
        return Ok(None);
    }
    trace!("{}/{}/{} this_z {:?} this_x {:?} this_y {:?}", z, x, y, this_z, this_x, this_y);
    let local_x = x % 8;
    let local_y = y % 8;
    let index = (local_x*size + local_y) as i64;
    trace!("{}/{}/{} index of wanted tile is {}", z, x, y, index);

    if index > (count as i64) {
        // This file doesn't have this many tiles
        trace!("{}/{}/{} There aren't that many tiles in this metatile, early return", z, x, y);
        return Ok(None)
    }

    input.seek(SeekFrom::Current(index*8))?;

    let offset = input.read_u32::<LittleEndian>()? as u64;
    let size = input.read_u32::<LittleEndian>()? as usize;
    trace!("{}/{}/{} byte offset is {} and size is {}", z, x, y, offset, size);

    if size == 0 {
        return Ok(None);
    }

    input.seek(SeekFrom::Start(offset))?;
    let mut data = vec![0u8; size];

    input.read_exact(&mut data)?;
    trace!("{}/{}/{} have read {} bytes. First 30: {:x?}", z, x, y, data.len(), data.iter().take(30).collect::<Vec<_>>());

    Ok(Some(data))
}
