use navymill::coordinate::Coordinate;
use navymill::coordinate::alphanum10::AlphaNum10;
use navymill::coordinate::format::CoordinateFormat;
use navymill::gamestate::ShotResult;
use navymill::gamestate::segmented_bitset::SegmentedBitSetGameState;
use navymill::layout::ShipLayoutProvider;
use navymill::layout::file::FileLayout;
use navymill::layout::random::RandomLayout;
use navymill::session::{FinishEvent, GameSession, SessionShootError};

use std::io::{self, Write};

fn main() -> io::Result<()> {
  let args: Vec<String> = std::env::args().collect();

  // Choose layout: FileLayout if a path is given, otherwise RandomLayout.
  let ship_coords: Vec<Coordinate<u8>> = if args.len() == 2 {
    // File-based layout
    let layout = FileLayout::new(&args[1])?;
    layout.positions_for("local")
  } else {
    // Random layout (10x10 board, classic fleet 5-4-3-3-2, no-touch, nondeterministic seed)
    let layout = RandomLayout {
      board_size: 10,
      ships: vec![5, 4, 3, 3, 2],
      no_touch: true,
      seed: None, // set Some(1234) for deterministic per-player placements
    };
    layout.positions_for("local")
  };

  // Create game state & session
  let state: SegmentedBitSetGameState = SegmentedBitSetGameState::new(ship_coords);
  let session = GameSession::new(state);

  println!("Game started! Enter coordinates like A1, B2, ..., J10 to shoot.");

  let stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut buffer = String::new();

  loop {
    print!("Your move: ");
    stdout.flush()?;
    buffer.clear();
    stdin.read_line(&mut buffer)?;

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
      continue;
    }

    match AlphaNum10::parse(trimmed) {
      Ok(coord) => match session.shoot(&coord) {
        Ok(result) => {
          match result {
            ShotResult::Hit => println!("💥 Hit!"),
            ShotResult::AlreadyHit => println!("⚠️ Already hit!"),
            ShotResult::Miss => println!("💨 Miss!"),
          }

          match session.maybe_finish() {
            FinishEvent::Finished { shots, duration } => {
              println!("🎉 All ships sunk in {shots} shots!");
              println!("⏱️ Duration: {:.2?}", duration);
              break;
            }
            FinishEvent::None => {}
          }
        }
        Err(SessionShootError::Finished) => {
          println!("⚠️ Game already finished, no more shots allowed.");
          break;
        }
      },
      Err(e) => {
        println!("❌ Invalid coordinate input: {e}");
        continue;
      }
    }
  }

  Ok(())
}
