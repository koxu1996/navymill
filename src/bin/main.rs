use navymill::coordinate::Coordinate;
use navymill::coordinate::alphanum10::AlphaNum10;
use navymill::coordinate::format::CoordinateFormat;
use navymill::gamestate::segmented_bitset::SegmentedBitSetGameState;
use navymill::gamestate::ShotResult;
use navymill::session::{FinishEvent, GameSession, SessionShootError};

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn read_ship_positions(path: &str) -> io::Result<Vec<Coordinate<u8>>> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);

  let mut coords = Vec::new();

  for line in reader.lines() {
    let line = line?;
    for token in line.trim().split(',') {
      if token.trim().is_empty() {
        continue;
      }
      let coord = AlphaNum10::parse(token.trim()).map_err(|e| {
        io::Error::new(
          io::ErrorKind::InvalidData,
          format!("Invalid coord {token}: {e}"),
        )
      })?;
      coords.push(coord);
    }
  }

  Ok(coords)
}

fn main() -> io::Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    eprintln!("Usage: {} <path_to_ships_file>", args[0]);
    std::process::exit(1);
  }

  let ship_coords = read_ship_positions(&args[1])?;
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
