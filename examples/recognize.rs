//! Prints the text in an image: `cargo run --example recognize -- page.png`
//!
//! With `--lines`, prints each line with where it is in the image, then each of its words.

use std::{env, process::ExitCode};

use image::open;
use native_ocr::{Rect, recognize_lines};

fn main() -> ExitCode {
	let args: Vec<String> = env::args().skip(1).collect();
	let show_lines = args.iter().any(|arg| arg == "--lines");
	let Some(path) = args.iter().find(|arg| !arg.starts_with("--")) else {
		eprintln!("usage: recognize <image> [--lines]");
		return ExitCode::FAILURE;
	};
	let image = match open(path) {
		Ok(image) => image.into_rgba8(),
		Err(err) => {
			eprintln!("couldn't open {path}: {err}");
			return ExitCode::FAILURE;
		}
	};
	match recognize_lines(image.as_raw(), image.width(), image.height()) {
		Ok(lines) => {
			for line in lines {
				if show_lines {
					println!("{} {}", describe(line.bounds()), line.text);
					for word in &line.words {
						println!("    {} {}", describe(word.bounds), word.text);
					}
				} else {
					println!("{}", line.text);
				}
			}
			ExitCode::SUCCESS
		}
		Err(err) => {
			eprintln!("{err}");
			ExitCode::FAILURE
		}
	}
}

fn describe(bounds: Rect) -> String {
	format!("[{:.0},{:.0} {:.0}x{:.0}]", bounds.x, bounds.y, bounds.width, bounds.height)
}
