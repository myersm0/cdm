use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
	pub history_path: PathBuf,
	pub coaccess_window: usize,
	pub path_aliases: HashMap<String, String>,
}

impl Default for AppConfig {
	fn default() -> Self {
		let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
		Self {
			history_path: home.join(".cd_history"),
			coaccess_window: 3,
			path_aliases: HashMap::new(),
		}
	}
}

impl AppConfig {
	pub fn config_path() -> PathBuf {
		dirs::config_dir()
			.unwrap_or_else(|| PathBuf::from("."))
			.join("cdm")
			.join("config.toml")
	}

	pub fn load() -> Self {
		let path = Self::config_path();
		if let Ok(contents) = std::fs::read_to_string(&path) {
			toml::from_str(&contents).unwrap_or_default()
		} else {
			Self::default()
		}
	}

	pub fn path_aliases(&self) -> Vec<(PathBuf, String)> {
		self.path_aliases
			.iter()
			.map(|(alias, path_str)| (PathBuf::from(path_str), alias.clone()))
			.collect()
	}
}
