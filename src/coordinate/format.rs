use super::Coordinate;

/// Trait for coordinate parsing/formatting strategies.
pub trait CoordinateFormat {
  type Err: std::error::Error;

  fn parse(s: &str) -> Result<Coordinate, Self::Err>;
  fn format(coord: Coordinate) -> String;
}
