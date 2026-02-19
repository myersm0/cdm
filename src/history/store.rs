use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn load_history(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
	let file = fs::File::open(path).await?;
	let reader = BufReader::new(file);
	let mut lines = reader.lines();
	let mut entries = Vec::new();
	while let Some(line) = lines.next_line().await? {
		let trimmed = line.trim().to_string();
		if !trimmed.is_empty() {
			entries.push(PathBuf::from(trimmed));
		}
	}
	Ok(entries)
}

pub async fn append_history(path: &Path, entry: &Path) -> Result<(), std::io::Error> {
	let mut file = fs::OpenOptions::new()
		.create(true)
		.append(true)
		.open(path)
		.await?;
	let mut line = entry.to_string_lossy().to_string();
	line.push('\n');
	file.write_all(line.as_bytes()).await?;
	Ok(())
}
