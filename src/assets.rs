// Copyright 2022 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod font {
	use iced::Font;

	const MONO_BYTES: &[u8] = include_bytes!(
		"../assets/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf"
	);

	pub(crate) const MONO: Font = Font::External {
		name: "JetBrains Mono",
		bytes: MONO_BYTES,
	};

	const BODY_BYTES: &[u8] =
		include_bytes!("../assets/fonts/Roboto/Roboto-Regular.ttf");

	pub(crate) const BODY: Font = Font::External {
		name: "Roboto",
		bytes: BODY_BYTES,
	};
}

pub(crate) mod icons {
	use iced::Font;

	const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/icons.ttf");

	pub(crate) const FONT: Font = Font::External {
		name: "Evalvana Icons",
		bytes: FONT_BYTES,
	};

	pub(crate) const NEW_CELL: char = '\u{e900}';

	pub(crate) const CLOSE_TAB: char = '\u{e901}';

	pub(crate) const EMPTY_TAB: char = '\u{e902}';
}

pub(crate) const ICON64: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/assets/icons/logo/logo_64.png"
));
