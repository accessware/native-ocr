# native-ocr

Text recognition with the OCR engine built into the operating system, so your app doesn't need to ship an OCR model or native library of its own.

Linux has no engine built in, so there it uses Tesseract from the user's package manager, such as `tesseract-ocr` and `tesseract-ocr-eng`. It's loaded when first used rather than linked, so your binary builds and runs without it, and recognizing returns `Error::NotInstalled` until it's installed. Set `TESSDATA_PREFIX` to use language data from somewhere other than the distro's usual directory.

| Platform | Engine | Languages |
|---|---|---|
| Windows 10 and 11 | `Windows.Media.Ocr` | The OCR language packs installed for the user's languages |
| macOS | Vision, at its accurate recognition level | Built into the OS |
| Linux | Tesseract 4 or 5, loaded at run time from the system's `libtesseract` | The Tesseract language data the user installed, matched to their locale, plus English |
| Everywhere else | None | |

## Usage

```rust
use native_ocr::{Error, recognize_rgba};

match recognize_rgba(&pixels, width, height) {
	Ok(text) => println!("{text}"),
	Err(Error::NoLanguage) => println!("Install an OCR language pack in Windows' language settings."),
	Err(Error::NotInstalled) => println!("Install Tesseract from your package manager."),
	Err(err) => println!("{err}"),
}
```

`pixels` is an RGBA8 bitmap: 4 bytes per pixel, rows top to bottom with no padding. The result is the recognized lines joined with newlines, in the order the engine reads them.

Recognition blocks, so call it from a worker thread in a GUI app. Use `max_image_dimension()` to render pages no larger than the engine works with, and `is_available()` to hide OCR features on platforms without an engine.

`recognize_lines` returns the same lines with each word's box in the image, in pixels from the top left. Engines read a line at a time and know nothing of the layout, so text in columns or boxes can come back interleaved, or with two columns read as one line. The boxes let you put it back together.

To try it on an image, with `--lines` to see the boxes:

```sh
cargo run --example recognize -- page.png --lines
```

## License

MIT
