//! `Windows.Media.Ocr`. Quality depends on the installed OCR language packs matching the text,
//! and the API has no preprocessing or accuracy settings.

use std::ptr::{addr_of_mut, copy_nonoverlapping, null_mut};

use windows::{
	Graphics::Imaging::{BitmapBufferAccessMode, BitmapPixelFormat, SoftwareBitmap},
	Media::Ocr::{OcrEngine, OcrLine},
	Win32::{
		Foundation::RPC_E_CHANGED_MODE,
		System::{
			Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
			WinRT::IMemoryBufferByteAccess,
		},
	},
	core::{Error as WinError, Interface},
};

use crate::{Error, Line, Rect, Result, Word};

pub const fn is_available() -> bool {
	true
}

/// What the engine reports, or the documented 2600 pixels if it can't be read.
pub fn max_image_dimension() -> u32 {
	OcrEngine::MaxImageDimension().unwrap_or(2600)
}

pub fn recognize_lines(rgba: &[u8], width: u32, height: u32) -> Result<Vec<Line>> {
	let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
	let initialized = hr.is_ok();
	if !initialized && hr != RPC_E_CHANGED_MODE {
		return Err(Error::Failed(format!("initializing COM: {}", WinError::from(hr))));
	}
	let result = recognize(rgba, width, height);
	if initialized {
		unsafe { CoUninitialize() };
	}
	result
}

fn recognize(rgba: &[u8], width: u32, height: u32) -> Result<Vec<Line>> {
	let engine = create_engine()?;
	let bitmap = build_bitmap(rgba, width, height)?;
	let result = engine.RecognizeAsync(&bitmap).and_then(|op| op.join()).map_err(|err| failed("recognizing", &err))?;
	let lines = result.Lines().map_err(|err| failed("reading the lines", &err))?;
	Ok(lines.into_iter().filter_map(|line| read_line(&line)).collect())
}

/// A line and its words, or `None` for one the engine can't hand back, which is left out rather than failing the whole image.
fn read_line(line: &OcrLine) -> Option<Line> {
	let text = line.Text().ok()?.to_string_lossy();
	let words = line
		.Words()
		.ok()?
		.into_iter()
		.filter_map(|word| {
			let bounds = word.BoundingRect().ok()?;
			Some(Word {
				text: word.Text().ok()?.to_string_lossy(),
				bounds: Rect { x: bounds.X, y: bounds.Y, width: bounds.Width, height: bounds.Height },
			})
		})
		.collect();
	Some(Line { text, words })
}

/// Creates an engine for the user's profile languages. The API gives no reason when it can't, so
/// a missing language pack is told apart by there being no recognizer languages at all.
fn create_engine() -> Result<OcrEngine> {
	OcrEngine::TryCreateFromUserProfileLanguages().map_err(|err| {
		let has_language = OcrEngine::AvailableRecognizerLanguages().is_ok_and(|langs| langs.Size().unwrap_or(0) > 0);
		if has_language { failed("creating the engine", &err) } else { Error::NoLanguage }
	})
}

fn build_bitmap(rgba: &[u8], width: u32, height: u32) -> Result<SoftwareBitmap> {
	let dimension = |value: u32| i32::try_from(value).map_err(|_| Error::InvalidImage("too large".to_string()));
	let bitmap = SoftwareBitmap::Create(BitmapPixelFormat::Rgba8, dimension(width)?, dimension(height)?)
		.map_err(|err| failed("creating the bitmap", &err))?;
	let buffer = bitmap.LockBuffer(BitmapBufferAccessMode::Write).map_err(|err| failed("locking the bitmap", &err))?;
	let reference = buffer.CreateReference().map_err(|err| failed("referencing the bitmap", &err))?;
	let access: IMemoryBufferByteAccess = reference.cast().map_err(|err| failed("accessing the bitmap", &err))?;
	let mut data: *mut u8 = null_mut();
	let mut capacity: u32 = 0;
	unsafe { access.GetBuffer(addr_of_mut!(data), addr_of_mut!(capacity)) }
		.map_err(|err| failed("reading the bitmap buffer", &err))?;
	if data.is_null() || capacity as usize != rgba.len() {
		return Err(Error::Failed(format!("bitmap buffer holds {capacity} bytes, expected {}", rgba.len())));
	}
	unsafe { copy_nonoverlapping(rgba.as_ptr(), data, rgba.len()) };
	// The lock and its references drop here, before the engine reads the bitmap.
	Ok(bitmap)
}

fn failed(step: &str, err: &WinError) -> Error {
	Error::Failed(format!("{step}: {err}"))
}
