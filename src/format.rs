use std::ops::RangeInclusive;

use lasso::{Interner, MiniSpur};

use crate::token::Token;
use crate::whitespace::Whitespace;
use crate::Tokens;

macro_rules! both_sides {
	($($inner:pat),+ $(,)?) => {
		($($inner)|+, _) | (_, $($inner)|+)
	};
}

#[derive(Debug, Clone)]
enum Action {
	Unformatted,
	Set(&'static str),
	SetIndentation(usize),
	SetIndentationRange(RangeInclusive<usize>),
	SetSpaces(usize),
	SetOneOrZeroSpaces,
}

impl Action {
	fn apply(
		self,
		interner: &mut impl Interner<MiniSpur>,
		whitespace: &mut Whitespace,
		source: &str,
		span: crate::span::Span,
	) {
		match self {
			Self::Unformatted => {
				eprintln!(
					"unformatted whitespace at span {}",
					crate::span::LineAndColumn::from_location_and_source(span.start, source)
				);
			}
			Self::Set(set) => {
				whitespace.set_static(interner, set);
			}
			Self::SetIndentation(indentation) => {
				whitespace.set_full(interner, None, Some(indentation..=indentation));
			}
			Self::SetIndentationRange(range) => {
				whitespace.set_full(interner, None, Some(range));
			}
			Self::SetSpaces(spaces) => {
				whitespace.set_full(interner, Some(spaces..=spaces), None);
			}
			Self::SetOneOrZeroSpaces => {
				if !whitespace.text(interner).is_empty() {
					whitespace.set_static(interner, " ");
				}
			}
		}
	}
}

pub fn format(interner: &mut impl Interner<MiniSpur>, tokens: &mut Tokens, source: &str) {
	for arounds in tokens.arounds_mut() {
		let (before, whitespace, after) = arounds.into_tuple();
		let action = match (before, after) {
			// files that are only whitespace
			(None, None) => Action::Set(""),
			// leading whitespace
			(None, Some((token, _))) => format_initial(token),
			// trailing whitespace
			(Some(_), None) => Action::Set("\n"),
			// normal whitespace
			(Some((before, _)), Some((after, _))) => format_single(before, after),
		};
		action.apply(
			interner,
			whitespace,
			source,
			crate::span::Span {
				start: before.map_or(0, |(_, span)| span.end),
				end: after.map_or(source.len().try_into().unwrap(), |(_, span)| span.start),
			},
		);
	}
}

fn format_initial(token: Token) -> Action {
	use Token as T;

	match token {
		T::Identifier => Action::Set("\t"),
		T::Error => unreachable!(),
		_ => Action::Set(""),
	}
}

fn format_single(before: Token, after: Token) -> Action {
	use Token as T;

	// clarity; one arm per rule and different rules may produce the same whitespace.
	#[allow(clippy::match_same_arms)]
	match (before, after) {
		(T::Identifier | T::Directive | T::LabelDeclaration, T::Colon) => Action::Set(""),
		(T::OpenRound | T::OpenSquare, _) | (_, T::CloseRound | T::CloseSquare) => Action::Set(""),
		(T::Comma, _) => Action::Set(" "),
		(_, T::Comma) => Action::Set(""),
		both_sides!(T::MiscBinaryOperator) => Action::Set(" "),
		(T::MiscUnaryOperator | T::MiscBinaryOrUnaryOperator, T::MiscUnaryOperator) => Action::Set(""),
		(_, T::MiscUnaryOperator) => Action::Set(" "),
		(T::MiscUnaryOperator, _) => Action::Set(""),
		(_, T::MiscBinaryOrUnaryOperator) => Action::Set(" "),
		(T::MiscBinaryOrUnaryOperator, _) => Action::SetOneOrZeroSpaces,
		(_, T::LabelDeclaration) => Action::SetIndentation(0),
		(_, T::Identifier | T::String) => Action::SetIndentation(1),
		(_, T::Directive) => Action::SetIndentationRange(0..=1),
		(_, T::LineComment) => Action::SetSpaces(2),
		(T::Directive | T::Identifier, _) => Action::SetSpaces(1),
		both_sides!(T::BlockComment) => Action::SetSpaces(1),
		both_sides!(T::Error) => unreachable!(),
		_ => Action::Unformatted,
	}
}
