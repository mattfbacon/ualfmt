use std::fmt::{self, Display, Formatter};

pub type Location = u32;

/// start..end
#[derive(Debug, Clone, Copy)]
pub struct Span {
	pub start: Location,
	pub end: Location,
}

impl From<std::ops::Range<usize>> for Span {
	fn from(range: std::ops::Range<usize>) -> Self {
		Self {
			start: range.start.try_into().unwrap(),
			end: range.end.try_into().unwrap(),
		}
	}
}

impl From<Span> for std::ops::Range<usize> {
	fn from(range: Span) -> Self {
		Self {
			start: range.start.try_into().unwrap(),
			end: range.end.try_into().unwrap(),
		}
	}
}

impl std::ops::Index<Span> for str {
	type Output = str;

	fn index(&self, index: Span) -> &Self::Output {
		&self[std::ops::Range::<usize>::from(index)]
	}
}

impl std::ops::Index<Span> for String {
	type Output = str;

	fn index(&self, index: Span) -> &Self::Output {
		&self[std::ops::Range::<usize>::from(index)]
	}
}

#[derive(Debug, Clone, Copy)]
pub struct LineAndColumn {
	/// Zero-indexed.
	pub line: Location,
	/// Zero-indexed.
	pub column: Location,
}

impl LineAndColumn {
	pub fn from_location_and_source(location: Location, source: &str) -> Self {
		let location_usize: usize = location.try_into().unwrap();
		let (line, line_start) = source
			.bytes()
			.enumerate()
			.filter(|&(_byte_idx, byte)| byte == b'\n')
			.enumerate()
			.map(|(line_idx, (byte_idx, _byte))| (line_idx + 1, byte_idx + 1))
			.take_while(|&(_line_idx, byte_idx)| byte_idx <= location_usize)
			.last()
			.unwrap_or((0, 0));
		let line = line.try_into().unwrap();
		let column = location - Location::try_from(line_start).unwrap();
		Self { line, column }
	}
}

impl Display for LineAndColumn {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"{line}:{column}",
			line = self.line + 1,
			column = self.column + 1,
		)
	}
}
