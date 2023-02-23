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

use std::fmt::{self, Display, Formatter};
use std::io::{Read as _, Write as _};

fn main() {
	let mut source = String::new();
	std::io::stdin().lock().read_to_string(&mut source).unwrap();

	let formatted = match ualfmt::format(&source) {
		Ok(formatted) => formatted,
		Err(error) => {
			eprintln!(
				"lexer error: something's wrong at {}, specifically the input {:?}",
				LineAndColumn::from_location_and_source(error.location, &source),
				error.problem_text
			);
			return;
		}
	};

	std::io::stdout()
		.lock()
		.write_all(formatted.as_bytes())
		.unwrap();
}

#[derive(Debug, Clone, Copy)]
struct LineAndColumn {
	/// Zero-indexed.
	line: u32,
	/// Zero-indexed.
	column: u32,
}

impl LineAndColumn {
	fn from_location_and_source(location: u32, source: &str) -> Self {
		let location_usize: usize = location.try_into().unwrap();
		let (line, line_start) = source
			.bytes()
			.enumerate()
			.filter(|&(_byte_idx, byte)| byte == b'\n')
			.enumerate()
			.map(|(line_idx, (byte_idx, _byte))| (line_idx + 1, byte_idx + 1))
			.take_while(|&(_line_idx, byte_idx)| byte_idx <= location_usize)
			.last()
			.unwrap_or((0, 0));
		let line = line.try_into().unwrap();
		let column = location - u32::try_from(line_start).unwrap();
		Self { line, column }
	}
}

impl Display for LineAndColumn {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		write!(
			formatter,
			"{line}:{column}",
			line = self.line + 1,
			column = self.column + 1,
		)
	}
}
