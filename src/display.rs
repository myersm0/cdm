use std::path::{Path, PathBuf};

pub fn format_path(
	path: &Path,
	home_dir: &Option<PathBuf>,
	aliases: &[(PathBuf, String)],
	max_width: usize,
) -> String {
	let mut display = path.to_string_lossy().to_string();
	let mut alias_matched = false;

	for (prefix, alias) in aliases {
		if let Ok(rest) = path.strip_prefix(prefix) {
			display = format!("[{}]/{}", alias, rest.display());
			alias_matched = true;
			break;
		}
	}

	if !alias_matched {
		if let Some(home) = home_dir {
			if let Ok(rest) = path.strip_prefix(home) {
				display = format!("~/{}", rest.display());
			}
		}
	}

	if display.len() > max_width {
		let components: Vec<&str> = display.split('/').collect();
		if components.len() > 3 {
			let first = components[0];
			let last_two = &components[components.len() - 2..];
			let truncated = format!("{}/…/{}", first, last_two.join("/"));
			if truncated.len() < display.len() {
				display = truncated;
			}
		}
		if display.len() > max_width {
			display = format!("…{}", &display[display.len() - max_width + 1..]);
		}
	}

	display
}
