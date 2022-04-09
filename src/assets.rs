// Copyright 2021 ThatsNoMoon
// Licensed under the Open Software License version 3.0

pub(crate) mod font {
	use iced::Font;

	pub(crate) const MONO_BYTES: &[u8] = include_bytes!(
		"../assets/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf"
	);

	pub(crate) const MONO: Font = Font::External {
		name: "JetBrains Mono",
		bytes: MONO_BYTES,
	};

	pub(crate) const BODY_BYTES: &[u8] =
		include_bytes!("../assets/fonts/Roboto/Roboto-Regular.ttf");

	pub(crate) const BODY: Font = Font::External {
		name: "Roboto",
		bytes: BODY_BYTES,
	};
}
