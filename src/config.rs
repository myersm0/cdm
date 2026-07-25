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
		let mut config = if let Ok(contents) = std::fs::read_to_string(&path) {
			toml::from_str(&contents).unwrap_or_default()
		} else {
			Self::default()
		};
		config.history_path = expand_home(config.history_path);
		config
	}

	pub fn path_aliases(&self) -> Vec<(PathBuf, String)> {
		self.path_aliases
			.iter()
			.map(|(alias, path_str)| (expand_home(PathBuf::from(path_str)), alias.clone()))
			.collect()
	}
}

fn expand_home(path: PathBuf) -> PathBuf {
	if let Ok(rest) = path.strip_prefix("~") {
		if let Some(home) = dirs::home_dir() {
			if rest.as_os_str().is_empty() {
				return home;
			}
			return home.join(rest);
		}
	}
	path
}

#[cfg(test)]
mod tests {
	use super::expand_home;
	use std::path::PathBuf;

	#[test]
	fn tilde_prefix_expands_to_home() {
		let home = dirs::home_dir().unwrap();
		assert_eq!(expand_home(PathBuf::from("~/.cd_history")), home.join(".cd_history"));
		assert_eq!(expand_home(PathBuf::from("~")), home);
	}

	#[test]
	fn other_paths_unchanged() {
		assert_eq!(expand_home(PathBuf::from("/absolute/path")), PathBuf::from("/absolute/path"));
		assert_eq!(expand_home(PathBuf::from("relative/path")), PathBuf::from("relative/path"));
		assert_eq!(expand_home(PathBuf::from("~user/path")), PathBuf::from("~user/path"));
	}
}
