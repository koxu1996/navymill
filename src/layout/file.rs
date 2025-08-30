use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::coordinate::alphanum10::AlphaNum10;
use crate::coordinate::format::CoordinateFormat;

use super::Coordinate;
use super::ShipLayoutProvider;

/// Reads a fixed layout once from disk; every player gets the same positions.
/// File format: tokens separated by commas and/or whitespace, e.g.:
///   A1, A2, A3
///   B5 C7 D10
pub struct FileLayout {
  positions: Vec<Coordinate<u8>>,
}

impl FileLayout {
  pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut positions = Vec::new();

    for line in reader.lines() {
      let line = line?;
      for token in line.split(|c: char| c.is_ascii_whitespace() || c == ',') {
        let t = token.trim();
        if t.is_empty() {
          continue;
        }
        let coord = AlphaNum10::parse(t).map_err(|e| {
          io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid coord {t}: {e}"),
          )
        })?;
        positions.push(coord);
      }
    }

    Ok(Self { positions })
  }
}

impl ShipLayoutProvider for FileLayout {
  fn positions_for(&self, _player: &str) -> Vec<Coordinate<u8>> {
    self.positions.clone()
  }
}
