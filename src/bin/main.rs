use std::io::{self, Write};
use std::sync::Arc;

use navymill::coordinate::alphanum10::AlphaNum10;
use navymill::coordinate::format::CoordinateFormat;
use navymill::gamestate::ShotResult;
use navymill::layout::ShipLayoutProvider;
use navymill::layout::file::FileLayout;
use navymill::layout::random::RandomLayout;
use navymill::manager::GameManager;
use navymill::session::SessionShootError;

fn main() -> io::Result<()> {
  let args: Vec<String> = std::env::args().collect();

  // Pick layout: file if path provided, else random.
  let layout: Arc<dyn ShipLayoutProvider> = if args.len() == 2 {
    Arc::new(FileLayout::new(&args[1])?)
  } else {
    Arc::new(RandomLayout {
      board_size: 2,
      ships: vec![1],
      no_touch: true,
      seed: None, // set Some(1234) for deterministic per-player boards
    })
  };

  let mgr = GameManager::new(layout);

  println!("Game started!");
  println!("• Single-player: type coordinates like `A1` (player defaults to `local`).");
  println!("• Multi-player:   type `PlayerName A1`.");
  println!("• Leaderboard:    `:top 10`");
  println!("• Quit:           `:q`");

  let stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut buffer = String::new();

  loop {
    print!("> ");
    stdout.flush()?;
    buffer.clear();
    if stdin.read_line(&mut buffer)? == 0 {
      break;
    }
    let line = buffer.trim();
    if line.is_empty() {
      continue;
    }

    // Commands
    if line == ":q" || line == ":quit" || line == ":exit" {
      break;
    }
    if let Some(rest) = line.strip_prefix(":top") {
      let n = rest.trim().parse::<usize>().unwrap_or(10);
      let entries = mgr.top(n);
      if entries.is_empty() {
        println!("(leaderboard empty)");
      } else {
        for (i, (player, shots, dur)) in entries.into_iter().enumerate() {
          println!(
            "{:>2}. {:<16} shots: {:<3} time: {:?}",
            i + 1,
            player,
            shots,
            dur
          );
        }
      }
      continue;
    }

    // Parse input as either "Player Coord" or just "Coord" (defaults to player "local")
    let mut parts = line.split_whitespace();
    let first = parts.next().unwrap();
    let (player, coord_str) = if let Some(second) = parts.next() {
      (first, second) // "Player Coord"
    } else {
      ("local", first) // single token -> default player
    };

    match AlphaNum10::parse(coord_str) {
      Ok(coord) => match mgr.shoot(player, coord) {
        Ok(result) => {
          match result {
            ShotResult::Hit => println!("{}: 💥 Hit!", player),
            ShotResult::AlreadyHit => println!("{}: ⚠️ Already hit!", player),
            ShotResult::Miss => println!("{}: 💨 Miss!", player),
          }

          // If this shot finished the game, manager has tombstoned it—report summary.
          if let Some(summary) = mgr.finished_summary(player) {
            println!(
              "🎉 {} finished in {} shots (⏱️ {:?})",
              summary.player, summary.shots, summary.duration
            );
          }
        }
        Err(SessionShootError::Finished) => {
          println!("✅ {} has already finished. No more shots allowed.", player);
        }
      },
      Err(e) => {
        println!("❌ Invalid coordinate input `{}`: {}", coord_str, e);
        continue;
      }
    }
  }

  Ok(())
}
