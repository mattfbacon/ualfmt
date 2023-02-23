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

impl Whitespace {
	pub fn is_empty(self) -> bool {
		self == Self::Empty
	}

	pub fn from_text(text: &str) -> Self {
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

	/*
	pub fn set_full(
		&mut self,
		interner: &mut impl Interner<MiniSpur>,
		spaces_range: Option<RangeInclusive<u8>>,
		indentation_range: Option<RangeInclusive<u8>>,
	) {
		enum OtherRepr {
			Spaces(u8),
			NewlinesAndIndentation { newlines: u8, indentation: u8 },
		}

		impl From<Whitespace> for OtherRepr {
			fn from(whitespace: Whitespace) -> Self {
				match whitespace {
					Whitespace::Empty => Self::Spaces(0),
					Whitespace::OneSpace => Self::Spaces(1),
					Whitespace::TwoSpaces => Self::Spaces(2),
					Whitespace::Tab => Self::NewlinesAndIndentation {
						newlines: 0,
						indentation: 1,
					},
					Whitespace::Newline => Self::NewlinesAndIndentation {
						newlines: 1,
						indentation: 0,
					},
					Whitespace::NewlineTab => Self::NewlinesAndIndentation {
						newlines: 1,
						indentation: 1,
					},
					Whitespace::TwoNewlines => Self::NewlinesAndIndentation {
						newlines: 2,
						indentation: 0,
					},
					Whitespace::TwoNewlinesTab => Self::NewlinesAndIndentation {
						newlines: 2,
						indentation: 1,
					},
				}
			}
		}

		impl From<OtherRepr> for Whitespace {
			fn from(repr: OtherRepr) -> Self {
				match repr {

				}
			}
		}

		let spaces_range = spaces_range.unwrap_or(0..=1);
		let indentation_range = indentation_range.unwrap_or(0..=1);

		let mut repr = OtherRepr::from(*self);
		match repr {
			Self::Spaces(spaces) => {
				*spaces = (*spaces).clamp_range(spaces_range);
			}
			Self::NewlinesAndIndentation {
				newlines,
				indentation,
			} => {
				*newlines = (*newlines).with_max(2);
				*indentation = (*indentation).clamp_range(indentation_range);
			}
		}
		*self = repr.into();
	}
	*/

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
}
