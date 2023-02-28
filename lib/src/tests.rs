macro_rules! tests {
	($($name:ident($source:tt)),* $(,)?) => {
		$(
			#[test]
			fn $name() {
				let input = include_str!(concat!("../../tests/", $source, ".in.s"));
				let expected = include_str!(concat!("../../tests/", $source, ".out.s"));
				let actual = crate::format(input).unwrap();
				assert_eq!(expected, actual);
			}
		)*
	};
}

tests! {
	test1("1"),
	test2("2"),
	test3("3"),
	test_block_comments("block-comments"),
}
