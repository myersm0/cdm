use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub use crate::display::format_path;

pub struct PickerItem {
	pub display: String,
	pub path: PathBuf,
}

pub struct PickerConfig {
	pub title: String,
	pub home_dir: Option<PathBuf>,
	pub path_aliases: Vec<(PathBuf, String)>,
	pub max_display_width: usize,
}

impl Default for PickerConfig {
	fn default() -> Self {
		Self {
			title: String::new(),
			home_dir: dirs::home_dir(),
			path_aliases: Vec::new(),
			max_display_width: 80,
		}
	}
}

pub fn terminal_width() -> usize {
	if let Ok(tty) = std::fs::File::open("/dev/tty") {
		let mut window_size: libc::winsize = unsafe { std::mem::zeroed() };
		let result = unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut window_size) };
		if result == 0 && window_size.ws_col > 0 {
			return window_size.ws_col as usize;
		}
	}
	std::env::var("COLUMNS")
		.ok()
		.and_then(|value| value.parse::<usize>().ok())
		.unwrap_or(80)
}

fn set_raw_mode(fd: i32) -> Option<libc::termios> {
	unsafe {
		let mut orig: libc::termios = std::mem::zeroed();
		if libc::tcgetattr(fd, &mut orig) != 0 {
			return None;
		}
		let mut raw = orig;
		raw.c_lflag &= !(libc::ICANON | libc::ECHO);
		raw.c_cc[libc::VMIN] = 1;
		raw.c_cc[libc::VTIME] = 0;
		if libc::tcsetattr(fd, libc::TCSANOW, &raw) != 0 {
			return None;
		}
		Some(orig)
	}
}

fn restore_mode(fd: i32, orig: &libc::termios) {
	unsafe {
		libc::tcsetattr(fd, libc::TCSANOW, orig);
	}
}

pub fn run_picker(items: &[PickerItem], config: &PickerConfig) -> Option<PathBuf> {
	if items.is_empty() {
		eprintln!("no results");
		return None;
	}

	let mut stderr = io::stderr();
	let count = items.len();
	let number_width = if count >= 100 { 3 } else if count >= 10 { 2 } else { 1 };

	if !config.title.is_empty() {
		writeln!(stderr, " {}", config.title).ok();
	}

	for (index, item) in items.iter().enumerate() {
		writeln!(
			stderr,
			" {:>width$}) {}",
			count - index,
			item.display,
			width = number_width,
		).ok();
	}
	writeln!(stderr).ok();
	write!(stderr, " go to (q to cancel): ").ok();
	stderr.flush().ok();

	let tty = match std::fs::File::open("/dev/tty") {
		Ok(f) => f,
		Err(_) => return None,
	};
	let fd = tty.as_raw_fd();
	let orig = match set_raw_mode(fd) {
		Some(t) => t,
		None => return None,
	};

	let mut buf = String::new();
	let mut reader = io::BufReader::new(&tty);
	let result = loop {
		let mut byte = [0u8; 1];
		if reader.read(&mut byte).unwrap_or(0) == 0 {
			break None;
		}
		let ch = byte[0] as char;

		match ch {
			'q' | 'Q' | '\x1b' => {
				writeln!(stderr).ok();
				break None;
			}
			'\r' | '\n' => {
				writeln!(stderr).ok();
				if buf.is_empty() {
					break None;
				}
				match buf.parse::<usize>() {
					Ok(n) if n >= 1 && n <= count => break Some(items[count - n].path.clone()),
					_ => break None,
				}
			}
			'0'..='9' => {
				let mut candidate = buf.clone();
				candidate.push(ch);
				let n = candidate.parse::<usize>().unwrap_or(0);

				if n < 1 || n > count {
					continue;
				}

				buf = candidate;
				write!(stderr, "{}", ch).ok();
				stderr.flush().ok();

				if n * 10 > count {
					writeln!(stderr).ok();
					break Some(items[count - n].path.clone());
				}
			}
			'\x7f' | '\x08' => {
				if buf.pop().is_some() {
					write!(stderr, "\x08 \x08").ok();
					stderr.flush().ok();
				}
			}
			_ => {}
		}
	};

	restore_mode(fd, &orig);
	result
}
