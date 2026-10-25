//! Text recognition with the OCR engine built into the operating system.
//!
//! - Windows: `Windows.Media.Ocr`, using the OCR language packs installed for the user's
//!   languages.
//! - macOS: Vision's accurate text recognition, whose languages ship with the OS.
//! - Linux: Tesseract, loaded at run time from the system's `libtesseract`, with the language data
//!   the user installed. Without the library, recognizing returns [`Error::NotInstalled`].
//! - Elsewhere: no engine, so [`is_available`] is `false` and recognizing returns
//!   [`Error::Unavailable`].
//!
//! Recognition blocks, so call it from a worker thread in a GUI app.

#[cfg_attr(target_os = "macos", path = "engine/macos.rs")]
#[cfg_attr(windows, path = "engine/windows.rs")]
#[cfg_attr(target_os = "linux", path = "engine/linux.rs")]
#[cfg_attr(not(any(windows, target_os = "macos", target_os = "linux")), path = "engine/unavailable.rs")]
mod engine;
mod error;
mod line;

pub use error::{Error, Result};
pub use line::{Line, Rect, Word};

/// Whether this system has an OCR engine. A `true` here can still fail with
/// [`Error::NoLanguage`] on Windows and Linux. On Linux this loads Tesseract the first time it's
/// called.
#[must_use]
#[allow(clippy::missing_const_for_fn, reason = "Linux looks for Tesseract at run time")]
pub fn is_available() -> bool {
	engine::is_available()
}

/// The largest image side the engine works with, or 0 without an engine. Engines downscale
/// anything bigger, so there's no point rendering pages larger than this.
#[must_use]
#[allow(clippy::missing_const_for_fn, reason = "Windows asks the OS at run time")]
pub fn max_image_dimension() -> u32 {
	engine::max_image_dimension()
}

/// Recognizes the text in an RGBA8 bitmap, returning the lines joined with newlines.
///
/// The bitmap is 4 bytes per pixel, rows top to bottom with no padding, so `width * height * 4`
/// bytes. Lines come in the order the engine reads them.
///
/// # Errors
///
/// [`Error::InvalidImage`] when the byte count doesn't match the size, [`Error::Unavailable`]
/// without an engine, [`Error::NotInstalled`] when Linux doesn't have Tesseract,
/// [`Error::NoLanguage`] when Windows has no OCR language pack for the user's languages or Linux
/// has no Tesseract language data, and [`Error::Failed`] when the engine fails.
pub fn recognize_rgba(rgba: &[u8], width: u32, height: u32) -> Result<String> {
	let lines = recognize_lines(rgba, width, height)?;
	Ok(lines.into_iter().map(|line| line.text).collect::<Vec<_>>().join("\n"))
}

/// Recognizes the text in an RGBA8 bitmap like [`recognize_rgba`], returning each line with where
/// its words are in the image.
///
/// Engines read a page a line at a time and know nothing of its layout, so text set in columns or
/// in boxes can come back with the lines of different columns in between each other, or with two
/// columns read as one line. The word boxes are what an app needs to put such text back together.
///
/// # Errors
///
/// As [`recognize_rgba`].
pub fn recognize_lines(rgba: &[u8], width: u32, height: u32) -> Result<Vec<Line>> {
	let expected = (width as usize) * (height as usize) * 4;
	if width == 0 || height == 0 || rgba.len() != expected {
		return Err(Error::InvalidImage(format!("{width}x{height} needs {expected} bytes, got {}", rgba.len())));
	}
	engine::recognize_lines(rgba, width, height)
}
