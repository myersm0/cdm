use std::path::{Path, PathBuf};

pub fn format_path(
	path: &Path,
	home_dir: &Option<PathBuf>,
	aliases: &[(PathBuf, String)],
	max_width: usize,
) -> String {
	use unicode_width::UnicodeWidthStr;

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

	if UnicodeWidthStr::width(display.as_str()) > max_width {
		let components: Vec<&str> = display.split('/').collect();
		if components.len() > 3 {
			let first = components[0];
			let last_two = &components[components.len() - 2..];
			let truncated = format!("{}/…/{}", first, last_two.join("/"));
			if UnicodeWidthStr::width(truncated.as_str()) < UnicodeWidthStr::width(display.as_str()) {
				display = truncated;
			}
		}
		if UnicodeWidthStr::width(display.as_str()) > max_width {
			display = truncate_keep_tail(&display, max_width);
		}
	}

	display
}

fn truncate_keep_tail(text: &str, max_width: usize) -> String {
	use std::collections::VecDeque;
	use unicode_width::UnicodeWidthChar;

	if max_width <= 1 {
		return "…".to_string();
	}
	let mut kept: VecDeque<char> = VecDeque::new();
	let mut used_width = 0;
	for character in text.chars().rev() {
		let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
		if used_width + character_width > max_width - 1 {
			break;
		}
		kept.push_front(character);
		used_width += character_width;
	}
	format!("…{}", kept.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn home_abbreviation() {
		let home = Some(PathBuf::from("/Users/alice"));
		let result = format_path(
			Path::new("/Users/alice/projects/foo"),
			&home,
			&[],
			200,
		);
		assert_eq!(result, "~/projects/foo");
	}

	#[test]
	fn no_home_match() {
		let home = Some(PathBuf::from("/Users/alice"));
		let result = format_path(
			Path::new("/opt/data/stuff"),
			&home,
			&[],
			200,
		);
		assert_eq!(result, "/opt/data/stuff");
	}

	#[test]
	fn alias_takes_priority_over_home() {
		let home = Some(PathBuf::from("/Users/alice"));
		let aliases = vec![
			(PathBuf::from("/Users/alice/projects"), "proj".to_string()),
		];
		let result = format_path(
			Path::new("/Users/alice/projects/bar/src"),
			&home,
			&aliases,
			200,
		);
		assert_eq!(result, "[proj]/bar/src");
	}

	#[test]
	fn middle_ellipsis_truncation() {
		let result = format_path(
			Path::new("/very/long/deeply/nested/path/to/some/dir"),
			&None,
			&[],
			200,
		);
		assert_eq!(result, "/very/long/deeply/nested/path/to/some/dir");

		let result = format_path(
			Path::new("/very/long/deeply/nested/path/to/some/dir"),
			&None,
			&[],
			20,
		);
		assert!(result.len() <= 20, "got len {}: {}", result.len(), result);
		assert!(result.contains('…'));
	}

	#[test]
	fn truncation_multibyte_no_panic() {
		let path_text = format!("/données/{}/café", "é".repeat(60));
		let result = format_path(Path::new(&path_text), &None, &[], 20);
		assert!(unicode_width::UnicodeWidthStr::width(result.as_str()) <= 20);
		assert!(result.starts_with('…'));
	}

	#[test]
	fn truncation_wide_characters() {
		let result = format_path(Path::new("/日本/語/の/深い/パス/名前"), &None, &[], 10);
		assert!(unicode_width::UnicodeWidthStr::width(result.as_str()) <= 10);
	}

	#[test]
	fn short_path_unchanged() {
		let result = format_path(
			Path::new("/a/b"),
			&None,
			&[],
			200,
		);
		assert_eq!(result, "/a/b");
	}
}
