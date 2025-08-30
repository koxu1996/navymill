use super::Coordinate;
use super::format::CoordinateFormat;

/// Alphanumeric coordinate format with rows A–J and columns 1–10 (zero-based internally).
pub struct AlphaNum10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlphaNumParseError {
  Empty,
  InvalidRow(char),
  MissingColumn,
  InvalidColumn(String),
  ColumnOutOfRange(u8),
}

impl std::fmt::Display for AlphaNumParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AlphaNumParseError::Empty => write!(f, "coordinate string is empty"),
      AlphaNumParseError::InvalidRow(c) => write!(f, "invalid row '{c}': expected A-J"),
      AlphaNumParseError::MissingColumn => write!(f, "missing column number"),
      AlphaNumParseError::InvalidColumn(s) => write!(f, "invalid column '{s}': expected 1-10"),
      AlphaNumParseError::ColumnOutOfRange(n) => {
        write!(f, "column {n} out of range: expected 1-10")
      }
    }
  }
}

impl std::error::Error for AlphaNumParseError {}

impl AlphaNum10 {
  fn parse_row(c: char) -> Result<u8, AlphaNumParseError> {
    match c {
      'A'..='J' => Ok((c as u8) - b'A'),
      _ => Err(AlphaNumParseError::InvalidRow(c)),
    }
  }

  fn parse_col(s: &str) -> Result<u8, AlphaNumParseError> {
    if s.is_empty() {
      return Err(AlphaNumParseError::MissingColumn);
    }

    s.parse::<u8>()
      .map_err(|_| AlphaNumParseError::InvalidColumn(s.to_string()))
      .and_then(|n| match n {
        1..=10 => Ok(n - 1), // Convert to 0-based
        _ => Err(AlphaNumParseError::ColumnOutOfRange(n)),
      })
  }
}

impl CoordinateFormat for AlphaNum10 {
  type CoordType = u8;
  type Err = AlphaNumParseError;
  /// Parse a coordinate string like "A1", "B2", "J10".
  ///
  /// Rows are represented by letters A–J (mapped to 0–9),
  /// and columns are represented by numbers 1–10 (mapped to 0–9).
  fn parse(s: &str) -> Result<Coordinate<Self::CoordType>, AlphaNumParseError> {
    let mut chars = s.chars();

    // Parse row (single letter A-J).
    let row_char = chars.next().ok_or(AlphaNumParseError::Empty)?;
    let row = Self::parse_row(row_char)?;

    // Parse column (remaining digits).
    let col_str: String = chars.collect();
    let col = Self::parse_col(&col_str)?;

    Ok(Coordinate { x: col, y: row })
  }

  /// Format a coordinate back to string like "A1", "B2", "J10".
  /// NOTE: This assumes the coordinate is valid for a 10x10 board.
  fn format(coord: Coordinate<Self::CoordType>) -> String {
    let row_char = (b'A' + coord.y) as char;
    let col = coord.x + 1;
    format!("{row_char}{col}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_coordinates() {
    assert_eq!(AlphaNum10::parse("A1").unwrap(), Coordinate { x: 0, y: 0 });
    assert_eq!(AlphaNum10::parse("J10").unwrap(), Coordinate { x: 9, y: 9 });
    assert_eq!(AlphaNum10::parse("E5").unwrap(), Coordinate { x: 4, y: 4 });
  }

  #[test]
  fn test_invalid_coordinates() {
    assert!(matches!(
      AlphaNum10::parse(""),
      Err(AlphaNumParseError::Empty)
    ));
    assert!(matches!(
      AlphaNum10::parse("K1"),
      Err(AlphaNumParseError::InvalidRow('K'))
    ));
    assert!(matches!(
      AlphaNum10::parse("a1"),
      Err(AlphaNumParseError::InvalidRow('a'))
    )); // case sensitive
    assert!(matches!(
      AlphaNum10::parse("A"),
      Err(AlphaNumParseError::MissingColumn)
    ));
    assert!(matches!(
      AlphaNum10::parse("A0"),
      Err(AlphaNumParseError::ColumnOutOfRange(0))
    ));
    assert!(matches!(
      AlphaNum10::parse("A11"),
      Err(AlphaNumParseError::ColumnOutOfRange(11))
    ));
    assert!(matches!(
      AlphaNum10::parse("Axx"),
      Err(AlphaNumParseError::InvalidColumn(_))
    ));
  }

  #[test]
  fn test_roundtrip() {
    for coord_str in ["A1", "J10", "E5"] {
      let coord = AlphaNum10::parse(coord_str).unwrap();
      assert_eq!(
        AlphaNum10::format(coord),
        coord_str,
        "roundtrip failed for {coord_str}"
      );
    }
  }
}
