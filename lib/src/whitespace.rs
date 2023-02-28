use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whitespace {
	Empty,
	OneSpace,
	TwoSpaces,
	Tab,
	Newline,
	NewlineTab,
	TwoNewlines,
	TwoNewlinesTab,
}

trait ClampExt: Ord + Copy {
	// ensures that the value is not more than `max`.
	fn with_max(self, max: Self) -> Self {
		std::cmp::min(self, max)
	}

	// ensures that the value is not less than `min`.
	fn with_min(self, min: Self) -> Self {
		std::cmp::max(self, min)
	}

	fn clamp_range(self, range: RangeInclusive<Self>) -> Self {
		self.with_max(*range.end()).with_min(*range.start())
	}
}

impl<T: Ord + Copy> ClampExt for T {}

fn from_text_hot(text: &str) -> Option<Whitespace> {
	use Whitespace as W;

	Some(match text {
		"" => W::Empty,
		" " => W::OneSpace,
		"  " => W::TwoSpaces,
		"\t" => W::Tab,
		"\n" => W::Newline,
		"\n\t" => W::NewlineTab,
		"\n\n" => W::TwoNewlines,
		"\n\n\t" => W::TwoNewlinesTab,
		_ => return None,
	})
}

impl Whitespace {
	#[must_use]
	pub fn from_text(text: &str) -> Self {
		if let Some(hot) = from_text_hot(text) {
			return hot;
		}

		let newlines = text.bytes().filter(|&b| b == b'\n').count();
		if newlines == 0 {
			let indentation = text.bytes().rev().take_while(|&b| b == b'\t').count();
			if indentation > 0 {
				Self::Tab
			} else {
				match text.len() {
					0 => Self::Empty,
					1 => Self::OneSpace,
					_ => Self::TwoSpaces,
				}
			}
		} else {
			let indentation = text.bytes().rev().take_while(|&b| b != b'\n').count();
			match (newlines, indentation) {
				(0, _) => unreachable!(),
				(1, 0) => Self::Newline,
				(_, 0) => Self::TwoNewlines,
				(1, _) => Self::NewlineTab,
				(_, _) => Self::TwoNewlinesTab,
			}
		}
	}

	#[must_use]
	pub fn text(self) -> &'static str {
		match self {
			Self::Empty => "",
			Self::OneSpace => " ",
			Self::TwoSpaces => "  ",
			Self::Tab => "\t",
			Self::Newline => "\n",
			Self::NewlineTab => "\n\t",
			Self::TwoNewlines => "\n\n",
			Self::TwoNewlinesTab => "\n\n\t",
		}
	}

	#[must_use]
	pub fn is_spaces(self) -> bool {
		matches!(self, Self::Empty | Self::OneSpace | Self::TwoSpaces)
	}

	pub fn set_indentation(&mut self, indentation: bool) {
		macro_rules! pairs {
			($($unindented:ident => $indented:ident),* $(,)?) => {
				match self {
					$(
						Self::$unindented => if indentation { *self = Self::$indented; },
						Self::$indented => if !indentation { *self = Self::$unindented; },
					)*
					Self::Tab => if !indentation { *self = Self::Empty; },
					Self::TwoSpaces => { *self = Self::OneSpace; }
					Self::OneSpace | Self::Empty => {}
				}
			};
		}

		pairs!(Newline => NewlineTab, TwoNewlines => TwoNewlinesTab);
	}

	#[must_use]
	pub fn indentation(self) -> Option<usize> {
		match self {
			Whitespace::Empty | Whitespace::OneSpace | Whitespace::TwoSpaces => None,
			Whitespace::Newline | Whitespace::TwoNewlines => Some(0),
			Whitespace::Tab | Whitespace::NewlineTab | Whitespace::TwoNewlinesTab => Some(1),
		}
	}
}
