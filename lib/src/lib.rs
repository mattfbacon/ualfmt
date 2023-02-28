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

use itertools::Itertools as _;
use logos::Logos as _;

use self::format::format3;
pub use self::span::Span;
use self::token::Token;

mod format;
mod span;
#[cfg(test)]
mod tests;
mod token;
mod whitespace;

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

	let mut output = String::with_capacity(source.len() / 2);

	let lexer = Token::lexer(source).spanned();

	let margin = std::iter::repeat(None).take(2);

	let core = lexer.map(|(token, span)| {
		let span: Span = span.into();
		Some(if token == Token::Error {
			Err(Error { span })
		} else {
			Ok((token, span))
		})
	});

	let iter = margin.clone().chain(core).chain(margin);

	iter
		.tuple_windows()
		.try_for_each(|(token1, token2, token3)| {
			let (token1, token2, token3) = (
				token1.transpose()?,
				token2.transpose()?,
				token3.transpose()?,
			);

			format3([token1, token2, token3], source, &mut output);

			Ok(())
		})?;

	Ok(output)
}
