use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub fn load_history(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
	let file = std::fs::File::open(path)?;
	let reader = BufReader::new(file);
	let mut entries = Vec::new();
	for line in reader.lines() {
		let line = line?;
		let trimmed = line.trim();
		if !trimmed.is_empty() {
			entries.push(PathBuf::from(trimmed));
		}
	}
	Ok(entries)
}

pub fn append_history(path: &Path, entry: &Path) -> Result<(), std::io::Error> {
	let mut file = std::fs::OpenOptions::new()
		.create(true)
		.append(true)
		.open(path)?;
	let mut line = entry.to_string_lossy().to_string();
	line.push('\n');
	file.write_all(line.as_bytes())
}
