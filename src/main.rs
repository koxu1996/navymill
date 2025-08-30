use navymill::coordinate::Coordinate;
use navymill::coordinate::alphanum10::AlphaNum10;
use navymill::coordinate::format::CoordinateFormat;
use navymill::gamestate::segmented_bitset::SegmentedBitSetGameState;
use navymill::gamestate::{GameState, ShotResult};

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
  let game = SegmentedBitSetGameState::new(ship_coords);

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
      Ok(coord) => {
        let result = game.shoot(&coord);
        match result {
          ShotResult::Hit => println!("💥 Hit!"),
          ShotResult::AlreadyHit => println!("⚠️ Already hit!"),
          ShotResult::Miss => println!("💨 Miss!"),
          ShotResult::InvalidPosition => println!("❌ Invalid position!"), // just in case
        }
      }
      Err(e) => {
        println!("❌ Invalid coordinate input: {e}");
        continue;
      }
    }

    if game.has_won() {
      println!("🎉 All ships sunk! You win!");
      break;
    }
  }

  Ok(())
}
