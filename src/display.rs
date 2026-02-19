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
	fn truncation_at_max_width() {
		let result = format_path(
			Path::new("/very/long/deeply/nested/path/to/some/dir"),
			&None,
			&[],
			20,
		);
		assert!(result.len() <= 20, "got len {}: {}", result.len(), result);
		assert!(result.starts_with('…'));
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
