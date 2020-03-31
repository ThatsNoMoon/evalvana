#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
	Insert(char),
	Backspace,
}