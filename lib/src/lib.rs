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
#![warn(
	clippy::pedantic,
	clippy::dbg_macro,
	clippy::print_stderr,
	clippy::print_stdout,
	clippy::use_debug
)]
#![deny(unsafe_code)]

use logos::Logos as _;

use self::interspersed::{AroundOrBetween, Interspersed};
use self::span::Location;
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
	assert!(source.len() < u32::MAX as usize);

	let mut lexer = Token::lexer(source).spanned();

	let mut tokens = Vec::with_capacity(source.len() / 5);

	let mut prev_end = 0;
	lexer.try_for_each(|(token, span)| {
		let span = span::Span::from(span);

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
		Ok(())
	})?;

	let last_whitespace = Whitespace::from_text(
		&source[Span {
			start: prev_end,
			end: source.len().try_into().unwrap(),
		}],
	);

	let mut tokens = Tokens::from_rest_and_last(tokens, last_whitespace);

	retag_labels(&mut tokens);

	let mut auxiliary_source = String::new();

	format::format(&mut tokens, source);

	format_block_comments(&mut tokens, source, &mut auxiliary_source);

	let mut output = String::with_capacity(source.len() / 2);
	write(&mut output, source, &auxiliary_source, &tokens);
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

fn count_indentation(s: &str) -> usize {
	s.bytes().take_while(|&b| b == b'\t').count()
}

fn format_block_comments(tokens: &mut Tokens, source: &str, auxiliary_source: &mut String) {
	let auxiliary_offset: Location = source.len().try_into().unwrap();
	tokens.betweens_mut().enumerate().for_each(|(idx, item)| {
		let (before, (Token::BlockComment, span), _after) = item else { return };

		let content = &source[*span];

		let Some(indentation) = before.indentation().or_else(|| (idx == 0 && before == Whitespace::Empty).then_some(0)) else { return };

		if content
			.split('\n')
			.skip(1)
			.all(|line| line.is_empty() || count_indentation(line) == indentation)
		{
			return;
		}

		let start = auxiliary_offset + Location::try_from(auxiliary_source.len()).unwrap();

		content.split('\n').enumerate().for_each(|(idx, line)| {
			let indentation = if idx == 0 { 0 } else { indentation };
			let line_start = count_indentation(line);
			let line = &line[line_start..];

			auxiliary_source.reserve(indentation + line.len() + 1);
			auxiliary_source.extend(std::iter::repeat('\t').take(indentation));
			auxiliary_source.push_str(line);
			auxiliary_source.push('\n');
		});

		// remove final newline
		auxiliary_source.truncate(auxiliary_source.len() - 1);

		let end = auxiliary_offset + Location::try_from(auxiliary_source.len()).unwrap();

		*span = Span { start, end };
	});
}

fn write(output: &mut String, source: &str, auxiliary_source: &str, tokens: &Tokens) {
	tokens.iter_all().for_each(|item| {
		let text = match item {
			AroundOrBetween::Around(whitespace) => whitespace.text(),
			AroundOrBetween::Between((_token, span)) => {
				if usize::try_from(span.start).unwrap() < source.len() {
					&source[*span]
				} else {
					&auxiliary_source[span.move_left(source.len().try_into().unwrap())]
				}
			}
		};
		*output += text;
	});
}
