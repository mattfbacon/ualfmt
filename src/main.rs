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

use std::io::Read as _;

use lasso::{MiniSpur, Resolver, Rodeo};
use logos::Logos as _;

use self::interspersed::{AroundOrBetween, Interspersed};
use self::span::{LineAndColumn, Span};
use self::token::Token;
use self::whitespace::Whitespace;

mod format;
mod interspersed;
mod span;
mod token;
mod whitespace;

type Tokens = Interspersed<Whitespace, (Token, Span)>;

fn main() {
	let mut interner = Rodeo::<MiniSpur>::new();

	let mut input = String::new();
	std::io::stdin().lock().read_to_string(&mut input).unwrap();

	let lexer = Token::lexer(&input)
		.spanned()
		.map(|(token, span)| (token, span::Span::from(span)));

	let mut tokens = Vec::new();

	let mut prev_end = 0;
	for (token, span) in lexer {
		if token == Token::Error {
			eprintln!(
				"cannot format, lexer error at {position}: {input:?}",
				position = LineAndColumn::from_location_and_source(span.start, &input),
				input = &input[span]
			);
			return;
		}

		let whitespace_before = Span {
			start: prev_end,
			end: span.start,
		};
		let whitespace_before = Whitespace::of(&mut interner, &input[whitespace_before]);
		tokens.push((whitespace_before, (token, span)));
		prev_end = span.end;
	}

	let last_whitespace = Whitespace::of(
		&mut interner,
		&input[Span {
			start: prev_end,
			end: input.len().try_into().unwrap(),
		}],
	);

	let mut tokens = Tokens::from_rest_and_last(tokens, last_whitespace);

	retag_labels(&mut tokens);

	format::format(&mut interner, &mut tokens, &input);

	write(std::io::stdout().lock(), &interner, &input, &tokens).unwrap();
}

fn retag_labels(tokens: &mut Tokens) {
	let mut betweens = tokens.betweens_mut().map(|(_, token, _)| token).peekable();
	while let Some((between, _)) = betweens.next() {
		if let (label, Some((Token::Colon, _))) = (between, betweens.peek()) {
			*label = Token::LabelDeclaration;
		}
	}
}

fn write(
	mut writer: impl std::io::Write,
	interner: &impl Resolver<MiniSpur>,
	source: &str,
	tokens: &Tokens,
) -> std::io::Result<()> {
	tokens.iter_all().try_for_each(|item| {
		let text = match item {
			AroundOrBetween::Around(whitespace) => whitespace.text(interner),
			AroundOrBetween::Between((_token, span)) => &source[*span],
		};
		writer.write_all(text.as_bytes())
	})
}
