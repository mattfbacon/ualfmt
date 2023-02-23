#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

use logos::Logos as _;

use self::interspersed::{AroundOrBetween, Interspersed};
pub use self::span::Span;
use self::token::Token;
use self::whitespace::Whitespace;

mod format;
mod interspersed;
mod span;
#[cfg(test)]
mod tests;
mod token;
mod whitespace;

type Tokens = Interspersed<Whitespace, (Token, Span)>;

#[derive(Debug, Clone, Copy)]
pub struct Error {
	pub span: Span,
}

/// Format the given ARM assembly string.
///
/// # Errors
///
/// If the lexer is unable to lex the input.
///
/// # Panics
///
/// If the source is longer than about 4 GB (`u32::MAX`).
pub fn format(source: &str) -> Result<String, Error> {
	let lexer = Token::lexer(source)
		.spanned()
		.map(|(token, span)| (token, span::Span::from(span)));

	let mut tokens = Vec::new();

	let mut prev_end = 0;
	for (token, span) in lexer {
		if token == Token::Error {
			return Err(Error { span });
		}

		let whitespace_before = Span {
			start: prev_end,
			end: span.start,
		};
		let whitespace_before = Whitespace::from_text(&source[whitespace_before]);
		tokens.push((whitespace_before, (token, span)));
		prev_end = span.end;
	}

	let last_whitespace = Whitespace::from_text(
		&source[Span {
			start: prev_end,
			end: source.len().try_into().unwrap(),
		}],
	);

	let mut tokens = Tokens::from_rest_and_last(tokens, last_whitespace);

	retag_labels(&mut tokens);

	format::format(&mut tokens, source);

	let mut output = String::new();
	write(&mut output, source, &tokens);
	Ok(output)
}

fn retag_labels(tokens: &mut Tokens) {
	let mut betweens = tokens.betweens_mut().map(|(_, token, _)| token).peekable();
	while let Some((between, _)) = betweens.next() {
		if let (label, Some((Token::Colon, _))) = (between, betweens.peek()) {
			*label = Token::LabelDeclaration;
		}
	}
}

fn write(output: &mut String, source: &str, tokens: &Tokens) {
	tokens.iter_all().for_each(|item| {
		let text = match item {
			AroundOrBetween::Around(whitespace) => whitespace.text(),
			AroundOrBetween::Between((_token, span)) => &source[*span],
		};
		*output += text;
	});
}
