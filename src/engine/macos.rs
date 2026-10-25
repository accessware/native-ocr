//! Vision's text recognition. Vision ships its recognition languages with the OS, so there's no
//! language pack to install and no `NoLanguage` case in practice.

use std::{ptr::null, result::Result as StdResult};

use objc2::{
	AnyThread, msg_send,
	rc::{Retained, autoreleasepool},
};
use objc2_core_foundation::{CFData, CFRetained, CGRect};
use objc2_core_graphics::{
	CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
};
use objc2_foundation::{NSArray, NSDictionary, NSError, NSRange};
use objc2_vision::{
	VNImageRequestHandler, VNRecognizeTextRequest, VNRecognizedText, VNRectangleObservation, VNRequest,
	VNRequestTextRecognitionLevel,
};

use crate::{Error, Line, Rect, Result, Word};

pub const fn is_available() -> bool {
	true
}

/// About a 300 DPI letter page. Vision documents no limit, and larger pages only cost memory.
pub const fn max_image_dimension() -> u32 {
	4000
}

pub fn recognize_lines(rgba: &[u8], width: u32, height: u32) -> Result<Vec<Line>> {
	let image = cg_image(rgba, width, height)?;
	autoreleasepool(|_| recognize(&image, f64::from(width), f64::from(height)))
}

fn cg_image(rgba: &[u8], width: u32, height: u32) -> Result<CFRetained<CGImage>> {
	let data = CFData::from_bytes(rgba);
	let provider = CGDataProvider::with_cf_data(Some(&data)).ok_or_else(|| failed("creating the data provider"))?;
	let space = CGColorSpace::new_device_rgb().ok_or_else(|| failed("creating the color space"))?;
	let (width, height) = (width as usize, height as usize);
	unsafe {
		CGImage::new(
			width,
			height,
			8,
			32,
			width * 4,
			Some(&space),
			CGBitmapInfo(CGImageAlphaInfo::Last.0),
			Some(&provider),
			null(),
			false,
			CGColorRenderingIntent::RenderingIntentDefault,
		)
	}
	.ok_or_else(|| failed("creating the image"))
}

fn recognize(image: &CGImage, width: f64, height: f64) -> Result<Vec<Line>> {
	let request = VNRecognizeTextRequest::new();
	// The fast level is for short strings in camera frames, not pages of body text.
	request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
	request.setUsesLanguageCorrection(true);
	let handler = unsafe {
		VNImageRequestHandler::initWithCGImage_options(VNImageRequestHandler::alloc(), image, &NSDictionary::new())
	};
	let as_request: &VNRequest = &request;
	handler
		.performRequests_error(&NSArray::from_slice(&[as_request]))
		.map_err(|err| Error::Failed(err.localizedDescription().to_string()))?;
	let Some(observations) = request.results() else {
		return Ok(Vec::new());
	};
	Ok(observations
		.iter()
		.filter_map(|observation| observation.topCandidates(1).firstObject())
		.map(|candidate| {
			let text = candidate.string().to_string();
			let words = words(&text)
				.into_iter()
				.filter_map(|(word, range)| {
					Some(Word { text: word.to_string(), bounds: pixels(word_box(&candidate, range)?, width, height) })
				})
				.collect();
			Line { text, words }
		})
		.collect())
}

/// Where the characters of `range` are, normalized to the image with the origin at its lower left. The bindings leave this method out, so it is sent by name.
fn word_box(candidate: &VNRecognizedText, range: NSRange) -> Option<CGRect> {
	let result: StdResult<Retained<VNRectangleObservation>, Retained<NSError>> =
		unsafe { msg_send![candidate, boundingBoxForRange: range, error: _] };
	Some(unsafe { result.ok()?.boundingBox() })
}

/// Turns a Vision rectangle into one in pixels from the top left, as the other engines report them.
#[allow(clippy::cast_possible_truncation, reason = "an image is far smaller than f32 can count")]
fn pixels(rect: CGRect, width: f64, height: f64) -> Rect {
	Rect {
		x: (rect.origin.x * width) as f32,
		y: ((1.0 - rect.origin.y - rect.size.height) * height) as f32,
		width: (rect.size.width * width) as f32,
		height: (rect.size.height * height) as f32,
	}
}

/// The words of `text`, split at whitespace, each with its range in UTF-16 units, which is how Foundation counts a string.
fn words(text: &str) -> Vec<(&str, NSRange)> {
	let mut words = Vec::new();
	let mut start = None;
	let mut offset = 0;
	for (byte, ch) in text.char_indices() {
		if ch.is_whitespace() {
			if let Some((from, units)) = start.take() {
				words.push((&text[from..byte], NSRange::new(units, offset - units)));
			}
		} else if start.is_none() {
			start = Some((byte, offset));
		}
		offset += ch.len_utf16();
	}
	if let Some((from, units)) = start {
		words.push((&text[from..], NSRange::new(units, offset - units)));
	}
	words
}

fn failed(step: &str) -> Error {
	Error::Failed(format!("{step} failed"))
}
