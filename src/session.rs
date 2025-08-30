use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::coordinate::Coordinate;
use crate::gamestate::ShotResult;

use super::gamestate::GameState;

/// Error returned if someone tries to shoot a finished session.
#[derive(Debug, Copy, Clone)]
pub enum SessionShootError {
  Finished,
}

/// Indicates whether this shot finished the game.
#[derive(Debug, Copy, Clone)]
pub enum FinishEvent {
  None,
  /// Carries final stats captured atomically at finish time.
  Finished {
    shots: u32,
    duration: Duration,
  },
}

/// Represents a game session, tracking the game state, shots taken, and timing information.
pub struct GameSession<G: GameState + Send + Sync> {
  pub state: G,
  pub num_shots: AtomicU64,
  pub time_started: Instant,
  pub is_finished: AtomicBool,
}

impl<G: GameState + Send + Sync> GameSession<G> {
  pub fn new(state: G) -> Self {
    Self {
      state,
      num_shots: AtomicU64::new(0),
      time_started: Instant::now(),
      is_finished: AtomicBool::new(false),
    }
  }

  /// Fast check (lock-free).
  #[inline]
  pub fn finished(&self) -> bool {
    self.is_finished.load(Ordering::Relaxed)
  }

  /// Returns total shots so far.
  #[inline]
  pub fn shots(&self) -> u64 {
    self.num_shots.load(Ordering::Relaxed)
  }

  pub fn shoot(&self, coord: &Coordinate) -> Result<ShotResult, SessionShootError> {
    if self.finished() {
      return Err(SessionShootError::Finished);
    }

    let outcome = self.state.shoot(coord);
    self.num_shots.fetch_add(1, Ordering::Relaxed);
    Ok(outcome)
  }

  /// Check if the game is now won, and if so finalize exactly once.
  /// Returns `FinishEvent::Finished` with stats if this thread wins the race,
  /// or `FinishEvent::None` otherwise.
  pub fn maybe_finish(&self) -> FinishEvent {
    if self.state.has_won() {
      if self
        .is_finished
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
      {
        let duration = self.time_started.elapsed();
        let shots = self.num_shots.load(Ordering::Relaxed) as u32;
        return FinishEvent::Finished { shots, duration };
      }
    }
    FinishEvent::None
  }
}
