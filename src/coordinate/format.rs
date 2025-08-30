use super::Coordinate;

/// Trait for coordinate parsing/formatting strategies.
pub trait CoordinateFormat {
  type CoordType;
  type Err: std::error::Error;

  fn parse(s: &str) -> Result<Coordinate<Self::CoordType>, Self::Err>;
  fn format(coord: Coordinate<Self::CoordType>) -> String;
}
