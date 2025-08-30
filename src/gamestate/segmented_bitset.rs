use std::sync::atomic::Ordering;
use std::{collections::HashMap, sync::atomic::AtomicU64};

use super::Coordinate;
use super::GameState;
use super::ShotResult;

/// Game state implementation using segmented bitsets for tracking hits.
/// Highly memory efficient and supports lock-free concurrent access.
pub struct SegmentedBitSetGameState {
  /// Maps coordinate into (segment index, bit index).
  ship_positions: HashMap<Coordinate, (usize, u8)>,

  /// Stores hit status per segment.
  hit_segments: Vec<AtomicU64>,

  /// Total number of ship fields to hit.
  total_ship_fields: usize,
}

impl SegmentedBitSetGameState {
  /// Creates a new game state with the given ship positions.
  /// `ship_positions` is a list of coordinates where ships are located.
  pub fn new(ship_positions: impl IntoIterator<Item = Coordinate>) -> Self {
    let mut position_map = HashMap::new();
    let mut segments = Vec::new();

    for coord in ship_positions {
      let idx = position_map.len();
      let segment_idx = idx / 64;
      let bit_idx = (idx % 64) as u8;

      // Ensure the segments vector has enough segments.
      if segment_idx >= segments.len() {
        segments.push(AtomicU64::new(0));
      }

      position_map.insert(coord, (segment_idx, bit_idx));
    }

    Self {
      total_ship_fields: position_map.len(),
      ship_positions: position_map,
      hit_segments: segments,
    }
  }
}

impl GameState for SegmentedBitSetGameState {
  fn shoot(&self, coordinate: &Coordinate) -> ShotResult {
    // If the coordinate points to a ship, then it would have mapping to the segment and bit index.
    let (segment_idx, bit_idx) = match self.ship_positions.get(coordinate) {
      Some(v) => v,
      None => return ShotResult::Miss,
    };

    // Ship exists at this coordinate, try to mark it as hit.
    let mask = 1u64 << bit_idx;
    let prev = self.hit_segments[*segment_idx].fetch_or(mask, Ordering::AcqRel);
    if prev & mask != 0 {
      return ShotResult::AlreadyHit;
    }

    // New hit registered.
    ShotResult::Hit
  }

  fn has_won(&self) -> bool {
    let total_hits: usize = self
      .hit_segments
      .iter()
      .map(|seg| seg.load(Ordering::Acquire).count_ones() as usize)
      .sum();

    total_hits == self.total_ship_fields
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::coordinate::Coordinate;
  use crate::gamestate::{GameState, ShotResult};

  fn coord(x: u8, y: u8) -> Coordinate {
    Coordinate { x, y }
  }

  #[test]
  fn test_hit_and_miss() {
    let game = SegmentedBitSetGameState::new(vec![coord(1, 1), coord(2, 2)]);

    // Hits
    assert_eq!(game.shoot(&coord(1, 1)), ShotResult::Hit);
    assert_eq!(game.shoot(&coord(2, 2)), ShotResult::Hit);

    // Miss
    assert_eq!(game.shoot(&coord(0, 0)), ShotResult::Miss);
  }

  #[test]
  fn test_already_hit() {
    let game = SegmentedBitSetGameState::new(vec![coord(3, 3)]);

    assert_eq!(game.shoot(&coord(3, 3)), ShotResult::Hit);
    assert_eq!(game.shoot(&coord(3, 3)), ShotResult::AlreadyHit);
  }

  #[test]
  fn test_win_condition() {
    let game = SegmentedBitSetGameState::new(vec![coord(0, 0), coord(1, 0), coord(2, 0)]);

    assert!(!game.has_won());

    assert_eq!(game.shoot(&coord(0, 0)), ShotResult::Hit);
    assert!(!game.has_won());

    assert_eq!(game.shoot(&coord(1, 0)), ShotResult::Hit);
    assert!(!game.has_won());

    assert_eq!(game.shoot(&coord(2, 0)), ShotResult::Hit);
    assert!(game.has_won());
  }

  #[test]
  fn test_large_number_of_ships() {
    // Create 200 unique coordinates
    let positions: Vec<_> = (0..200).map(|i| coord(i % 10, i / 10)).collect();
    let game = SegmentedBitSetGameState::new(positions.clone());

    // Shoot all
    for pos in &positions {
      assert_eq!(game.shoot(pos), ShotResult::Hit);
    }

    // Win
    assert!(game.has_won());
  }
}
