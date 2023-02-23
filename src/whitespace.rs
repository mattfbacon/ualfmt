use std::ops::RangeInclusive;

use lasso::{Interner, MiniSpur, Resolver};

#[derive(Debug, Clone, Copy)]
pub struct Whitespace(MiniSpur);

fn make_spaces(spaces: usize) -> &'static str {
	&"    "[..spaces]
}

fn make_newlines_and_indentation(newlines: usize, indentation: usize) -> &'static str {
	assert!(newlines < 4);
	assert!(indentation < 4);
	&"\n\n\n\n\t\t\t\t"[(4 - newlines)..(4 + indentation)]
}

fn clamp(range: RangeInclusive<usize>, value: usize) -> usize {
	value.max(*range.start()).min(*range.end())
}

impl Whitespace {
	pub fn of(interner: &mut impl Interner<MiniSpur>, s: &str) -> Self {
		Self(interner.get_or_intern(s))
	}

	pub fn of_static(interner: &mut impl Interner<MiniSpur>, s: &'static str) -> Self {
		Self(interner.get_or_intern_static(s))
	}

	pub fn text(self, interner: &impl Resolver<MiniSpur>) -> &str {
		interner.resolve(&self.0)
	}

	pub fn set_static(&mut self, interner: &mut impl Interner<MiniSpur>, s: &'static str) {
		*self = Self::of_static(interner, s);
	}

	pub fn set_full(
		&mut self,
		interner: &mut impl Interner<MiniSpur>,
		spaces_range: Option<RangeInclusive<usize>>,
		indentation_range: Option<RangeInclusive<usize>>,
	) {
		let spaces_range = spaces_range.unwrap_or(0..=1);
		let indentation_range = indentation_range.unwrap_or(0..=1);

		let text = self.text(interner);
		let newlines = text.bytes().filter(|&b| b == b'\n').count();
		if newlines == 0 {
			let spaces = text.len();

			let spaces = clamp(spaces_range, spaces);
			self.set_static(interner, make_spaces(spaces));
		} else {
			let newlines = newlines.min(2);
			let indentation = text.bytes().rev().take_while(|&b| b == b'\t').count();
			let indentation = clamp(indentation_range, indentation);
			let text = make_newlines_and_indentation(newlines, indentation);
			self.set_static(interner, text);
		}
	}
}
