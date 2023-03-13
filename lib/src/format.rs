#![allow(clippy::module_name_repetitions)]

pub use crate::span::Span;
use crate::token::Token;
use crate::whitespace::Whitespace;

macro_rules! both_sides {
	($($inner:pat),+ $(,)?) => {
		($($inner)|+, _) | (_, $($inner)|+)
	};
}

fn unformatted(old: &str) -> Whitespace {
	Whitespace::from_text(old)
}

fn set_if_spaces(old: &str, set: Whitespace) -> Whitespace {
	let ws = Whitespace::from_text(old);
	if ws.is_spaces() {
		set
	} else {
		ws
	}
}

fn set_one_or_zero_spaces(old: &str) -> Whitespace {
	if old.is_empty() {
		Whitespace::Empty
	} else {
		Whitespace::OneSpace
	}
}

fn set_indentation(old: &str, indentation: bool) -> Whitespace {
	let mut ws = Whitespace::from_text(old);
	ws.set_indentation(indentation);
	ws
}

fn count_indentation(s: &str) -> usize {
	s.bytes().take_while(|&b| b == b'\t').count()
}

pub fn format3(
	[token1, token2, token3]: [Option<(Token, Span)>; 3],
	source: &str,
	output: &mut String,
) {
	match (token1, token2, token3) {
		(None, Some((mut first, first_span)), peek) => {
			if peek.map(|(peek, _span)| peek) == Some(Token::Colon) {
				first = Token::LabelDeclaration;
			}

			let leading = &source[..first_span.start as usize];
			let leading = format_leading(leading, first);
			output.push_str(leading);

			post_process(true, leading, first, &source[first_span], output);
		}
		(Some((mut token1, span1)), Some((mut token2, span2)), peek) => {
			if token2 == Token::Colon {
				token1 = Token::LabelDeclaration;
			}
			if peek.map(|(peek, _span)| peek) == Some(Token::Colon) {
				token2 = Token::LabelDeclaration;
			}

			let between = &source[span1.end as usize..span2.start as usize];
			let between = format_between(between, token1, token2);
			output.push_str(between);

			post_process(false, between, token2, &source[span2], output);
		}
		(Some(_last), None, None) => {
			output.push('\n');
		}
		_ => {}
	}
}

fn format_leading(_leading: &str, token: Token) -> &str {
	use Token as T;

	match token {
		T::Identifier => "\t",
		T::Error => unreachable!(),
		_ => "",
	}
}

fn format_between(between: &str, before: Token, after: Token) -> &str {
	use {Token as T, Whitespace as W};

	// clarity; one arm per rule and different rules may produce the same whitespace.
	#[allow(clippy::match_same_arms)]
	let new = match (before, after) {
		(T::Identifier | T::Directive | T::LabelDeclaration, T::Colon) => W::Empty,
		(T::OpenRound | T::OpenSquare, _) | (_, T::CloseRound | T::CloseSquare) => W::Empty,
		(T::Comma, _) => W::OneSpace,
		(_, T::Comma) => W::Empty,
		both_sides!(T::MiscBinaryOperator) => W::OneSpace,
		(T::MiscUnaryOperator | T::MiscBinaryOrUnaryOperator, T::MiscUnaryOperator) => W::Empty,
		(_, T::MiscUnaryOperator) => W::OneSpace,
		(T::MiscUnaryOperator, _) => W::Empty,
		(_, T::MiscBinaryOrUnaryOperator) => W::OneSpace,
		(T::MiscBinaryOrUnaryOperator, _) => set_one_or_zero_spaces(between),
		(_, T::LabelDeclaration) => set_indentation(between, false),
		(_, T::Identifier | T::String) => set_indentation(between, true),
		(_, T::LineComment) => set_if_spaces(between, W::TwoSpaces),
		(T::Directive | T::Identifier, _) => set_if_spaces(between, W::OneSpace),
		(_, T::Directive) => unformatted(between),
		both_sides!(T::BlockComment) => set_if_spaces(between, W::OneSpace),
		both_sides!(T::Error) => unreachable!(),
		_ => unformatted(between),
	};

	new.text()
}

fn post_process(first: bool, before: &str, token: Token, token_text: &str, out: &mut String) {
	if token != Token::BlockComment {
		out.push_str(token_text);
		return;
	};

	if let Some(indentation) = Whitespace::from_text(before)
		.indentation()
		.or_else(|| first.then_some(0))
	{
		token_text.split('\n').enumerate().for_each(|(idx, line)| {
			let indentation = if idx == 0 { 0 } else { indentation };
			let line_start = count_indentation(line);
			let line = &line[line_start..];

			out.reserve(indentation + line.len() + 1);
			out.extend(std::iter::repeat('\t').take(indentation));
			out.push_str(line);
			out.push('\n');
		});

		// remove final newline
		out.truncate(out.len() - 1);
	} else {
		out.reserve(token_text.len());

		token_text.split('\n').for_each(|line| {
			out.push_str(line);
			out.push(' ');
		});

		// remove final space
		out.truncate(out.len() - 1);
	};
}
