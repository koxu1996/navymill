use crate::coordinate::Coordinate;

mod segmented_bitset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotResult {
  Hit,
  AlreadyHit,
  Miss,
  InvalidPosition,
}

/// Trait representing the state of a battleship game.
pub trait GameState {
  // NOTE: We enforce the implementation to use internal mutability.
  fn shoot(&self, coordinate: &Coordinate) -> ShotResult;

  // Checks if all ships have been sunk.
  fn has_won(&self) -> bool;
}
