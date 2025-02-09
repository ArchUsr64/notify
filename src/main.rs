use crate::app::App;
use atty::Stream;
use fonts::Font;
use log::{debug, error};
use std::env;
use std::str::FromStr;
use std::time::{Duration, Instant};
use window::Window;

mod app;
mod fonts;
mod window;

const FONT_SIZE: usize = 30;
const DEFAULT_DELAY: Duration = Duration::from_secs(5);

fn main() {
	env_logger::init();

	let font = Font::from_pbm(include_bytes!("./res/font_atlas.pbm"), FONT_SIZE).unwrap();
	let mut args = env::args();
	let popup_delay = args
		.nth(1)
		.and_then(|i| i.parse::<u64>().ok())
		.map(Duration::from_secs)
		.unwrap_or_else(|| {
			error!(
				r#"Invalid usage
Correct Usage: notify <time in seconds>
eg: 'notify 5'"#
			);
			DEFAULT_DELAY
		});

	let display_text = if atty::isnt(Stream::Stdin) {
		let mut buf = String::new();
		let stdin = std::io::stdin();
		stdin.read_line(&mut buf).unwrap();
		buf
	} else {
		String::from_str(
			"[Placeholder text] pipe into notify to display custom text eg: 'echo hello | notify'",
		)
		.unwrap()
	};
	debug!(
		"Popup Delay: {:?}, Display Text: '{display_text}'",
		popup_delay
	);

	let (mut window, mut event_queue) = Window::new(
		(display_text.len() * FONT_SIZE / 2) as u32,
		// +2 for rendering the top and bottom borders (1px each)
		FONT_SIZE as u32 + 2,
		App::new(font, display_text),
	);

	let start = Instant::now();
	loop {
		event_queue.blocking_dispatch(&mut window).unwrap();

		if start.elapsed() > popup_delay {
			break;
		}
	}

	window.app.exit();
}
