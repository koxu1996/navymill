use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};
use std::time::Instant;

use navymill::coordinate::Coordinate;
use navymill::layout::ShipLayoutProvider;
use navymill::layout::file::FileLayout;
use navymill::layout::random::RandomLayout;
use navymill::manager::GameManager;
use navymill::session::SessionShootError;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn coord_new(row: u8, col: u8) -> Coordinate<u8> {
  // If you have a different constructor, change here:
  Coordinate { x: col, y: row }
}

fn main() {
  // CLI: simulate [players] [threads] [file|random] [path?]
  let args: Vec<String> = std::env::args().collect();
  let (players, threads, layout_kind, path) = match args.as_slice() {
    // cargo run --bin simulate 200 8 random
    [_, p, t, k] => (
      p.parse::<usize>().unwrap_or(200),
      t.parse::<usize>().unwrap_or(8),
      k.as_str(),
      None,
    ),
    // cargo run --bin simulate 200 8 file ships.txt
    [_, p, t, k, file] => (
      p.parse::<usize>().unwrap_or(200),
      t.parse::<usize>().unwrap_or(8),
      k.as_str(),
      Some(file.as_str()),
    ),
    _ => {
      eprintln!(
        "Usage:\n  {} <players> <threads> <random|file> [path_to_ships_file]",
        args.get(0).map(String::as_str).unwrap_or("simulate")
      );
      std::process::exit(1);
    }
  };

  // Choose layout
  let layout: Arc<dyn ShipLayoutProvider> = match (layout_kind, path) {
    ("file", Some(p)) => Arc::new(FileLayout::new(p).expect("failed to read layout file")),
    ("random", _) => Arc::new(RandomLayout {
      board_size: 10,
      ships: vec![5, 4, 3, 3, 2],
      no_touch: true,
      seed: Some(0xC0FFEE), // make runs reproducible; set None for fully random
    }),
    _ => {
      eprintln!("layout must be 'random' or 'file' (with path)");
      std::process::exit(1);
    }
  };

  let mgr = Arc::new(GameManager::new(layout));
  let board_size: u8 = 10; // keep in sync with your layout (RandomLayout::board_size)

  // Prepare player names
  let names: Vec<String> = (1..=players).map(|i| format!("Player{:04}", i)).collect();

  // Distribute work via atomic index
  let idx = Arc::new(AtomicUsize::new(0));

  let start = Instant::now();

  crossbeam_utils::thread::scope(|scope| {
    for tid in 0..threads {
      let mgr = Arc::clone(&mgr);
      let idx = Arc::clone(&idx);
      let names = &names;

      scope.spawn(move |_| {
        // Thread-local RNG
        let mut rng = StdRng::seed_from_u64(0xBEEF_0000_0000_0000 ^ (tid as u64));

        loop {
          let i = idx.fetch_add(1, Ordering::Relaxed);
          if i >= names.len() {
            break;
          }
          let player = &names[i];

          // Each bot fires random coordinates until the manager reports Finished.
          loop {
            let r = rng.random_range(0..board_size);
            let c = rng.random_range(0..board_size);
            let coord = coord_new(r, c);

            match mgr.shoot(player, coord) {
              Ok(_outcome) => {
                // (Optional) react to ShotResult if you’d like
                // match _outcome { ShotResult::Hit => ..., _ => {} }
              }
              Err(SessionShootError::Finished) => break, // done with this player
            }
          }
        }
      });
    }
  })
  .expect("thread scope failed");

  let elapsed = start.elapsed();

  // Print results
  println!(
    "Completed {} games using {} threads in {:.2?}",
    players, threads, elapsed
  );

  // Leaderboard top 15
  let top = mgr.top(15);
  if top.is_empty() {
    println!("(leaderboard is empty?)");
  } else {
    println!("Top 15:");
    for (i, (player, shots, dur)) in top.into_iter().enumerate() {
      println!(
        "{:>2}. {:<12} shots: {:<3} time: {:?}",
        i + 1,
        player,
        shots,
        dur
      );
    }
  }
}
