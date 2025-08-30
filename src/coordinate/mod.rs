mod alphanum10;
mod format;

/// Represents a coordinate on a battleship board.
///
/// The `x` field indicates the column, and the `y` field indicates the row.
///
/// NOTE: Storing coordinates with `u8` allows representing points on boards
/// up to 256×256 in size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate<T = u8> {
  pub x: T,
  pub y: T,
}
