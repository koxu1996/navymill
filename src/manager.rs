use std::{sync::Arc, time::Duration};

use crossbeam_skiplist::SkipMap;
use dashmap::DashMap;

use crate::{
  coordinate::Coordinate,
  gamestate::{ShotResult, segmented_bitset::SegmentedBitSetGameState},
  layout::ShipLayoutProvider,
  session::{FinishEvent, GameSession, SessionShootError},
};

/// Pin the concrete board + session types so the manager isn't generic.
type Board = SegmentedBitSetGameState;
type Session = GameSession<Board>;

/// Compact record we keep once a player finishes (frees board memory).
#[derive(Clone, Debug)]
pub struct FinishedSummary {
  pub player: String,
  pub shots: u32,
  pub duration: Duration,
}

/// One entry in the sessions map: either running or tombstoned.
pub enum Slot {
  Active(Arc<Session>),
  Finished(FinishedSummary),
}

/// (shots ASC, duration_ns ASC, player ASC)
fn lb_key(player: &str, shots: u32, duration: Duration) -> (u32, u128, String) {
  let ns = duration.as_secs() as u128 * 1_000_000_000u128 + duration.subsec_nanos() as u128;
  (shots, ns, player.to_string())
}

pub struct GameManager {
  sessions: DashMap<String, Slot>, // player → Active | Finished
  leaderboard: SkipMap<(u32, u128, String), ()>, // ordered keys, empty value
  layout: Arc<dyn ShipLayoutProvider>,
}

impl GameManager {
  pub fn new(layout: Arc<dyn ShipLayoutProvider>) -> Self {
    Self {
      sessions: DashMap::new(),
      leaderboard: SkipMap::new(),
      layout,
    }
  }

  /// First shot creates a session for this player (case-sensitive name).
  /// Returns the outcome of the shot or an error if the game already finished.
  pub fn shoot(
    &self,
    player: &str,
    coord: Coordinate<u8>,
  ) -> Result<ShotResult, SessionShootError> {
    let player_key = player.to_string();

    // --- Phase 1: obtain Arc<Session> without keeping any guard alive
    let session: Arc<Session> = {
      match self.sessions.entry(player_key.clone()) {
        dashmap::Entry::Vacant(v) => {
          let board = Board::new(self.layout.positions_for(player));
          let arc = Arc::new(Session::new(board));
          // insert Active and drop the entry guard immediately
          v.insert(Slot::Active(arc.clone()));
          arc
        }
        dashmap::Entry::Occupied(o) => {
          // DON'T try to deref `o`. Use `o.get()` to see the value.
          match o.get() {
            Slot::Finished(_) => return Err(SessionShootError::Finished),
            Slot::Active(s) => Arc::clone(s),
          }
        }
      }
    }; // guard dropped here

    // --- Phase 2: operate on the session (no map guard held)
    let outcome = session.shoot(&coord)?;

    if let FinishEvent::Finished { shots, duration } = session.maybe_finish() {
      // publish to leaderboard (independent structure)
      self.leaderboard.insert(lb_key(player, shots, duration), ());
      // Re-enter the map to replace with a tombstone. No guard from before is alive.
      match self.sessions.entry(player_key) {
        dashmap::Entry::Occupied(mut e) => {
          e.insert(Slot::Finished(FinishedSummary {
            player: player.to_string(),
            shots,
            duration,
          }));
        }
        dashmap::Entry::Vacant(e) => {
          // If someone removed it (shouldn't happen), just insert the tombstone.
          e.insert(Slot::Finished(FinishedSummary {
            player: player.to_string(),
            shots,
            duration,
          }));
        }
      }
    }

    Ok(outcome)
  }

  /// Return top-N (player, shots, duration) according to leaderboard order.
  pub fn top(&self, n: usize) -> Vec<(String, u32, Duration)> {
    self
      .leaderboard
      .iter()
      .take(n)
      .map(|e| {
        let (shots, ns, player) = e.key().clone();
        let secs = (ns / 1_000_000_000) as u64;
        let nanos = (ns % 1_000_000_000) as u32;
        (player, shots, Duration::new(secs, nanos))
      })
      .collect()
  }

  /// Has this player's game finished?
  pub fn is_finished(&self, player: &str) -> bool {
    self
      .sessions
      .get(player)
      .map(|s| matches!(&*s, Slot::Finished(_)))
      .unwrap_or(false)
  }

  /// Count currently active players.
  pub fn active_players(&self) -> usize {
    self
      .sessions
      .iter()
      .filter(|s| matches!(&*s.value(), Slot::Active(_)))
      .count()
  }

  /// (Optional) Inspect a finished player's summary.
  pub fn finished_summary(&self, player: &str) -> Option<FinishedSummary> {
    self.sessions.get(player).and_then(|s| match &*s {
      Slot::Finished(f) => Some(f.clone()),
      _ => None,
    })
  }
}
