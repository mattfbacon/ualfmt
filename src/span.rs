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
