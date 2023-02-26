pub type Location = u32;

/// start..end
#[derive(Debug, Clone, Copy)]
pub struct Span {
	pub start: Location,
	pub end: Location,
}

impl Span {
	#[must_use]
	pub fn move_left(self, by: u32) -> Self {
		Self {
			start: self.start - by,
			end: self.end - by,
		}
	}

	#[must_use]
	pub fn remove_prefix(self, amount: u32) -> Self {
		Self {
			start: self.start + amount,
			end: self.end,
		}
	}

	#[must_use]
	pub fn remove_suffix(self, amount: u32) -> Self {
		Self {
			start: self.start,
			end: self.end - amount,
		}
	}
}

impl From<std::ops::Range<usize>> for Span {
	#[allow(clippy::cast_possible_truncation)] // already checked externally
	fn from(range: std::ops::Range<usize>) -> Self {
		Self {
			start: range.start as u32,
			end: range.end as u32,
		}
	}
}

impl From<Span> for std::ops::Range<usize> {
	fn from(range: Span) -> Self {
		Self {
			start: range.start as usize,
			end: range.end as usize,
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
