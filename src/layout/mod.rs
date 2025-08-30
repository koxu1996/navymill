use crate::coordinate::Coordinate;

pub mod file;

/// Provide ship positions for a player on demand.
pub trait ShipLayoutProvider: Send + Sync {
  fn positions_for(&self, player: &str) -> Vec<Coordinate<u8>>;
}
