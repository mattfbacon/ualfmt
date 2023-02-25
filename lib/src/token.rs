#[derive(logos::Logos, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
	#[regex(r"\s+", logos::skip)]
	#[error]
	Error,

	#[regex(r"(//|# )[^\n]*")]
	LineComment,
	#[regex(r"/\*([^*]|\*[^/])*\*/")]
	BlockComment,

	#[regex(r"\.[a-zA-Z_][a-zA-Z_0-9]*")]
	Directive,

	#[regex(r"[a-zA-Z_][a-zA-Z_0-9.]*")]
	#[token(".")]
	Identifier,

	LabelDeclaration,

	#[regex(r"0[bB][01]+")]
	#[regex(r"0[oO][0-7]+")]
	#[regex(r"0[xX][0-9a-fA-F]+")]
	#[regex(r"[0-9]+(\.[0-9]*([eE][-+]?[0-9]+)?)?")]
	#[regex(r"\.[0-9]+([eE][-+]?[0-9]+)?")]
	#[regex(r"[0-9]+[eE][-+]?[0-9]+")]
	Number,

	#[regex(r#""((\\([\\"]|\n[ \t]*))|[^"])*""#)]
	String,

	#[token(":")]
	Colon,
	#[token(",")]
	Comma,
	#[token("|")]
	#[token("<<")]
	#[token(">>")]
	#[token("&")]
	#[token("+")]
	#[token("*")]
	#[token("/")]
	MiscBinaryOperator,
	#[token("~")]
	#[token("#")]
	#[token("=")]
	#[token(r"\")]
	MiscUnaryOperator,
	#[token("-")]
	MiscBinaryOrUnaryOperator,

	#[token("(")]
	OpenRound,
	#[token(")")]
	CloseRound,
	#[token("[")]
	OpenSquare,
	#[token("]")]
	CloseSquare,
}
