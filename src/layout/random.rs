use std::hash::{DefaultHasher, Hash, Hasher};

use rand::{Rng, SeedableRng, rng, rngs::StdRng};

use crate::{coordinate::Coordinate, layout::ShipLayoutProvider};

/// Randomly places ships on an NxN board.
/// - `ships`: lengths, e.g. `[5,4,3,3,2]`
/// - `no_touch`: if true, ships can't touch even diagonally
/// - `seed`: if Some, per-player deterministic; if None, non-deterministic
pub struct RandomLayout {
  pub board_size: u8,
  pub ships: Vec<u8>,
  pub no_touch: bool,
  pub seed: Option<u64>,
}
impl RandomLayout {
  pub fn new(board_size: u8, ships: Vec<u8>) -> Self {
    Self {
      board_size,
      ships,
      no_touch: false,
      seed: None,
    }
  }

  fn rng_for(&self, player: &str) -> StdRng {
    if let Some(base) = self.seed {
      // deterministic per-player
      let mut h = DefaultHasher::new();
      base.hash(&mut h);
      player.hash(&mut h);
      StdRng::seed_from_u64(h.finish())
    } else {
      // nondeterministic: pull a fresh seed from thread_rng
      let seed: u64 = rng().random();
      StdRng::seed_from_u64(seed)
    }
  }

  #[inline]
  fn in_bounds(&self, r: i16, c: i16) -> bool {
    r >= 0 && c >= 0 && (r as u8) < self.board_size && (c as u8) < self.board_size
  }

  fn overlaps_or_touches(
    &self,
    occupied: &Vec<(u8, u8)>,
    candidate: &[(u8, u8)],
    no_touch: bool,
  ) -> bool {
    for &(r, c) in candidate {
      if occupied.iter().any(|&(ro, co)| ro == r && co == c) {
        return true;
      }
      if no_touch {
        // check 8-neighborhood to forbid touching
        for dr in -1..=1 {
          for dc in -1..=1 {
            let rr = r as i16 + dr;
            let cc = c as i16 + dc;
            if rr < 0 || cc < 0 {
              continue;
            }
            let (rr, cc) = (rr as u8, cc as u8);
            if occupied.iter().any(|&(ro, co)| ro == rr && co == cc) {
              return true;
            }
          }
        }
      }
    }
    false
  }

  #[inline]
  fn coord_new(row: u8, col: u8) -> Coordinate<u8> {
    // If your Coordinate doesn't have a direct ctor, replace this with whatever you use, e.g.:
    // Coordinate::new(row, col)
    // or AlphaNum10::from_row_col(row, col).into()
    Coordinate { x: col, y: row }
  }
}
impl ShipLayoutProvider for RandomLayout {
  fn positions_for(&self, player: &str) -> Vec<Coordinate<u8>> {
    let mut rng = self.rng_for(player);
    let mut occupied: Vec<(u8, u8)> = Vec::new();

    for &len in &self.ships {
      'place: loop {
        let horizontal = rng.random_bool(0.5);
        let max_r = self.board_size as i16 - if horizontal { 1 } else { len as i16 };
        let max_c = self.board_size as i16 - if horizontal { len as i16 } else { 1 };
        if max_r < 0 || max_c < 0 {
          // ship too long for this board
          continue;
        }
        let r0 = rng.random_range(0..=max_r) as i16;
        let c0 = rng.random_range(0..=max_c) as i16;

        // Build candidate cells
        let mut cand: Vec<(u8, u8)> = Vec::with_capacity(len as usize);
        for i in 0..len {
          let (rr, cc) = if horizontal {
            (r0, c0 + i as i16)
          } else {
            (r0 + i as i16, c0)
          };
          if !self.in_bounds(rr, cc) {
            continue 'place;
          }
          cand.push((rr as u8, cc as u8));
        }

        if self.overlaps_or_touches(&occupied, &cand, self.no_touch) {
          continue 'place;
        }

        occupied.extend_from_slice(&cand);
        break 'place;
      }
    }

    // Convert to your Coordinate type
    occupied
      .into_iter()
      .map(|(r, c)| Self::coord_new(r, c))
      .collect()
  }
}
