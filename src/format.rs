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
	Unformatted { intentional: bool },
	Set(Whitespace),
	SetIndentation(bool),
	SetIfSpaces(Whitespace),
	/*
	SetIndentation(u8),
	SetIndentationRange(RangeInclusive<u8>),
	SetSpaces(u8),
	*/
	SetOneOrZeroSpaces,
}

impl Action {
	fn apply(self, whitespace: &mut Whitespace, source: &str, span: crate::span::Span) {
		match self {
			Self::Unformatted { intentional } => {
				if !intentional {
					eprintln!(
						"unformatted whitespace at span {}",
						crate::span::LineAndColumn::from_location_and_source(span.start, source)
					);
				}
			}
			Self::Set(set) => {
				*whitespace = set;
			}
			Self::SetIndentation(indentation) => {
				whitespace.set_indentation(indentation);
			}
			Self::SetIfSpaces(set) => {
				if whitespace.is_spaces() {
					*whitespace = set;
				}
			}
			/*
			Self::SetIndentation(indentation) => {
				whitespace.set_full(interner, None, Some(indentation..=indentation));
			}
			Self::SetIndentationRange(range) => {
				whitespace.set_full(interner, None, Some(range));
			}
			Self::SetSpaces(spaces) => {
				whitespace.set_full(interner, Some(spaces..=spaces), None);
			}
			*/
			Self::SetOneOrZeroSpaces => {
				if !whitespace.is_empty() {
					*whitespace = Whitespace::OneSpace;
				}
			}
		}
	}
}

pub fn format(tokens: &mut Tokens, source: &str) {
	for arounds in tokens.arounds_mut() {
		let (before, whitespace, after) = arounds.into_tuple();
		let action = match (before, after) {
			// files that are only whitespace
			(None, None) => Action::Set(Whitespace::Empty),
			// leading whitespace
			(None, Some((token, _))) => format_initial(token),
			// trailing whitespace
			(Some(_), None) => Action::Set(Whitespace::Newline),
			// normal whitespace
			(Some((before, _)), Some((after, _))) => format_single(before, after),
		};
		action.apply(
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
		T::Identifier => Action::Set(Whitespace::Tab),
		T::Error => unreachable!(),
		_ => Action::Set(Whitespace::Empty),
	}
}

fn format_single(before: Token, after: Token) -> Action {
	use {Token as T, Whitespace as W};

	// clarity; one arm per rule and different rules may produce the same whitespace.
	#[allow(clippy::match_same_arms)]
	match (before, after) {
		(T::Identifier | T::Directive | T::LabelDeclaration, T::Colon) => Action::Set(W::Empty),
		(T::OpenRound | T::OpenSquare, _) | (_, T::CloseRound | T::CloseSquare) => {
			Action::Set(W::Empty)
		}
		(T::Comma, _) => Action::Set(W::OneSpace),
		(_, T::Comma) => Action::Set(W::Empty),
		both_sides!(T::MiscBinaryOperator) => Action::Set(W::OneSpace),
		(T::MiscUnaryOperator | T::MiscBinaryOrUnaryOperator, T::MiscUnaryOperator) => {
			Action::Set(W::Empty)
		}
		(_, T::MiscUnaryOperator) => Action::Set(W::OneSpace),
		(T::MiscUnaryOperator, _) => Action::Set(W::Empty),
		(_, T::MiscBinaryOrUnaryOperator) => Action::Set(W::OneSpace),
		(T::MiscBinaryOrUnaryOperator, _) => Action::SetOneOrZeroSpaces,
		(_, T::LabelDeclaration) => Action::SetIndentation(false),
		(_, T::Identifier | T::String) => Action::SetIndentation(true),
		(_, T::Directive) => Action::Unformatted { intentional: true },
		(_, T::LineComment) => Action::SetIfSpaces(W::TwoSpaces),
		(T::Directive | T::Identifier, _) => Action::SetIfSpaces(W::OneSpace),
		both_sides!(T::BlockComment) => Action::SetIfSpaces(W::OneSpace),
		both_sides!(T::Error) => unreachable!(),
		_ => Action::Unformatted { intentional: false },
	}
}
