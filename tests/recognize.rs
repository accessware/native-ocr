use image::open;
use native_ocr::{Error, Line, Rect, Word, is_available, recognize_lines, recognize_rgba};

#[test]
fn reads_the_sample() {
	let image = open("tests/sample.png").expect("sample image").into_rgba8();
	match recognize_rgba(image.as_raw(), image.width(), image.height()) {
		Ok(text) => {
			let text = text.to_lowercase();
			assert!(text.contains("quick brown fox"), "{text}");
			assert!(text.contains("windows 10, 2026"), "{text}");
		}
		// CI machines may have no Tesseract, or an engine but no language to run it with.
		Err(Error::NoLanguage | Error::NotInstalled) => {}
		Err(Error::Unavailable) => assert!(!is_available()),
		Err(err) => panic!("{err}"),
	}
}

#[test]
fn places_the_sample_s_words() {
	let image = open("tests/sample.png").expect("sample image").into_rgba8();
	let size = |side: u32| f32::from(u16::try_from(side).expect("a small sample"));
	let (width, height) = (size(image.width()), size(image.height()));
	match recognize_lines(image.as_raw(), image.width(), image.height()) {
		Ok(lines) => {
			let line =
				lines.iter().find(|line| line.text.to_lowercase().contains("quick brown fox")).expect("the first line");
			let words: Vec<_> = line.words.iter().map(|word| word.text.to_lowercase()).collect();
			assert!(words.windows(2).any(|pair| pair == ["quick", "brown"]), "{words:?}");
			for pair in line.words.windows(2) {
				assert!(pair[0].bounds.x < pair[1].bounds.x, "words out of order: {pair:?}");
			}
			for word in &line.words {
				let bounds = word.bounds;
				assert!(bounds.width > 0.0 && bounds.height > 0.0, "{word:?}");
				assert!(bounds.x >= 0.0 && bounds.x + bounds.width <= width, "{word:?}");
				assert!(bounds.y >= 0.0 && bounds.y + bounds.height <= height, "{word:?}");
			}
			let second =
				lines.iter().find(|line| line.text.to_lowercase().contains("windows")).expect("the second line");
			assert!(second.bounds().y > line.bounds().y + line.bounds().height, "the second line is below the first");
		}
		Err(Error::NoLanguage | Error::NotInstalled) => {}
		Err(Error::Unavailable) => assert!(!is_available()),
		Err(err) => panic!("{err}"),
	}
}

#[test]
fn rejects_a_mismatched_buffer() {
	assert!(matches!(recognize_rgba(&[0; 12], 2, 2), Err(Error::InvalidImage(_))));
	assert!(matches!(recognize_rgba(&[], 0, 0), Err(Error::InvalidImage(_))));
	assert!(matches!(recognize_lines(&[0; 12], 2, 2), Err(Error::InvalidImage(_))));
}

#[test]
fn a_line_is_as_big_as_its_words() {
	let word = |x, y, width, height| Word { text: String::new(), bounds: Rect { x, y, width, height } };
	let line = Line { text: String::new(), words: vec![word(10.0, 20.0, 30.0, 10.0), word(50.0, 18.0, 20.0, 15.0)] };
	assert_eq!(line.bounds(), Rect { x: 10.0, y: 18.0, width: 60.0, height: 15.0 });
	assert_eq!(Line { text: String::new(), words: Vec::new() }.bounds(), Rect::default());
}
