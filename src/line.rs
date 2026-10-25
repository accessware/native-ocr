/// A rectangle in the image's pixels, measured from its top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
	pub x: f32,
	pub y: f32,
	pub width: f32,
	pub height: f32,
}

impl Rect {
	/// The smallest rectangle holding both.
	#[must_use]
	pub fn union(self, other: Self) -> Self {
		let (left, top) = (self.x.min(other.x), self.y.min(other.y));
		let right = (self.x + self.width).max(other.x + other.width);
		let bottom = (self.y + self.height).max(other.y + other.height);
		Self { x: left, y: top, width: right - left, height: bottom - top }
	}
}

/// One word the engine read, and where it is in the image.
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
	pub text: String,
	pub bounds: Rect,
}

/// One line the engine read: its text, and its words in reading order.
///
/// The text is the engine's own, which puts spaces between words only where the script uses them.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
	pub text: String,
	pub words: Vec<Word>,
}

impl Line {
	/// Where the line is in the image: the rectangle around all of its words.
	#[must_use]
	pub fn bounds(&self) -> Rect {
		self.words.iter().map(|word| word.bounds).reduce(Rect::union).unwrap_or_default()
	}
}
