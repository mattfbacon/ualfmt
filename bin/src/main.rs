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

use std::io::{Read as _, Write};
use std::path::PathBuf;

use ariadne::{Config, Label, Report, ReportKind, Source};

#[derive(argh::FromArgs)]
/// format ARM assembly.
struct Args {
	/// format the given files in-place.
	///
	/// if no files are provided, format from stdin to stdout.
	#[argh(positional)]
	files: Vec<PathBuf>,
}

fn main() {
	let Args { files } = argh::from_env();

	if files.is_empty() {
		let mut input = String::new();
		std::io::stdin().lock().read_to_string(&mut input).unwrap();

		format(&input, || std::io::stdout().lock());
	} else {
		for file in &files {
			format(&std::fs::read_to_string(file).unwrap(), || {
				std::fs::File::create(file).unwrap()
			});
		}
	}
}

fn format<W: Write>(input: &str, make_output: impl FnOnce() -> W) {
	let formatted = match ualfmt_lib::format(input) {
		Ok(formatted) => formatted,
		Err(error) => {
			Report::build(ReportKind::Error, (), error.span.start.try_into().unwrap())
				.with_message("could not lex the input")
				.with_label(
					Label::new(std::ops::Range::<usize>::from(error.span))
						.with_message("the problematic input"),
				)
				.with_config(Config::default().with_tab_width(2))
				.finish()
				.eprint(Source::from(input))
				.unwrap();
			return;
		}
	};

	make_output().write_all(formatted.as_bytes()).unwrap();
}
