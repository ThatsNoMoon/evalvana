// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
	Insert(char),
	Backspace,
}
