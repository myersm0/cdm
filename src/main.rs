mod config;
mod display;
mod history;
mod inline;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::AppConfig;
use crate::history::coaccess::CoAccessGraph;
use crate::history::store;
use crate::inline::picker::{self, PickerItem, PickerConfig};

#[derive(Parser)]
#[command(
	name = "cdm",
	about = "cd with memory — fast filesystem navigation",
	after_help = "Use via shell wrappers (source shell/cdm.sh) so that \
		selections actually cd into the chosen directory.",
)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// List directories ahead of cwd
	Goahead {
		/// How many levels deep to search
		#[arg(short, long, default_value_t = 3)]
		depth: usize,
		/// Maximum results to show
		#[arg(short, long, default_value_t = 15)]
		number: usize,
	},
	/// Pick from most recently visited directories
	Cdr {
		/// Filter results by regex
		#[arg(short, long)]
		regex: Option<String>,
		/// Maximum results to show
		#[arg(short, long, default_value_t = 15)]
		number: usize,
		/// Only show directories under cwd
		#[arg(short, long)]
		prefix: bool,
		/// Only consider the N most recent history entries
		#[arg(short = 'H', long, default_value_t = 500)]
		history_depth: usize,
	},
	/// Pick from most frequently visited directories
	Cdf {
		/// Filter results by regex
		#[arg(short, long)]
		regex: Option<String>,
		/// Maximum results to show
		#[arg(short, long, default_value_t = 15)]
		number: usize,
		/// Only consider the N most recent history entries
		#[arg(short = 'H', long, default_value_t = 500)]
		history_depth: usize,
	},
	/// Pick from statistically co-accessed directories (NPMI)
	Cdp {
		/// Maximum results to show
		#[arg(short, long, default_value_t = 15)]
		number: usize,
	},
}

async fn list_directories(root: &PathBuf, max_depth: usize) -> Vec<PathBuf> {
	let mut result = Vec::new();
	let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
	queue.push_back((root.clone(), 0));

	while let Some((dir, depth)) = queue.pop_front() {
		if depth >= max_depth {
			continue;
		}
		let mut reader = match tokio::fs::read_dir(&dir).await {
			Ok(r) => r,
			Err(_) => continue,
		};
		let mut children = Vec::new();
		while let Ok(Some(entry)) = reader.next_entry().await {
			let path = entry.path();
			if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
				children.push(path);
			}
		}
		children.sort_by(|a, b| {
			a.file_name().map(|n| n.to_ascii_lowercase())
				.cmp(&b.file_name().map(|n| n.to_ascii_lowercase()))
		});
		for child in children {
			result.push(child.clone());
			queue.push_back((child, depth + 1));
		}
	}
	result
}

fn make_picker_config(title: &str, config: &AppConfig) -> PickerConfig {
	let width = std::env::var("COLUMNS")
		.ok()
		.and_then(|s| s.parse::<usize>().ok())
		.unwrap_or(80);
	PickerConfig {
		title: title.to_string(),
		home_dir: dirs::home_dir(),
		path_aliases: config.path_aliases(),
		max_display_width: width.saturating_sub(8),
	}
}

fn make_picker_items(paths: &[PathBuf], picker_config: &PickerConfig) -> Vec<PickerItem> {
	paths
		.iter()
		.map(|path| PickerItem {
			display: picker::format_path(
				path,
				&picker_config.home_dir,
				&picker_config.path_aliases,
				picker_config.max_display_width,
			),
			path: path.clone(),
		})
		.collect()
}

fn filter_paths(paths: Vec<PathBuf>, regex: &Option<String>, prefix: bool) -> Vec<PathBuf> {
	let cwd = std::env::current_dir().ok();
	let regex = regex.as_ref().and_then(|r| regex::Regex::new(r).ok());

	paths
		.into_iter()
		.filter(|path| {
			if let Some(ref re) = regex {
				if !re.is_match(&path.to_string_lossy()) {
					return false;
				}
			}
			if prefix {
				if let Some(ref cwd) = cwd {
					if !path.starts_with(cwd) {
						return false;
					}
				}
			}
			true
		})
		.collect()
}

fn tail(history: &[PathBuf], depth: usize) -> &[PathBuf] {
	let start = history.len().saturating_sub(depth);
	&history[start..]
}

fn pick_and_print(paths: &[PathBuf], title: &str, config: &AppConfig) -> Option<PathBuf> {
	let picker_config = make_picker_config(title, config);
	let items = make_picker_items(paths, &picker_config);
	let selected = picker::run_picker(&items, &picker_config)?;
	println!("{}", selected.display());
	Some(selected)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();
	let config = AppConfig::load();

	match cli.command {
		Commands::Goahead { depth, number } => {
			let cwd = std::fs::canonicalize(".")?;
			let dirs = list_directories(&cwd, depth).await;
			let limited: Vec<PathBuf> = dirs.into_iter().take(number).collect();

			if let Some(path) = pick_and_print(&limited, "goahead", &config) {
				store::append_history(&config.history_path, &path).await.ok();
			}
		}
		Commands::Cdr { regex, number, prefix, history_depth } => {
			let history = store::load_history(&config.history_path)
				.await
				.unwrap_or_default();
			let recent = tail(&history, history_depth);

			let mut seen = HashSet::new();
			let paths: Vec<PathBuf> = recent.iter()
				.rev()
				.filter(|p| seen.insert((*p).clone()))
				.cloned()
				.collect();
			let filtered = filter_paths(paths, &regex, prefix);
			let limited: Vec<PathBuf> = filtered.into_iter().take(number).collect();

			if let Some(path) = pick_and_print(&limited, "recent", &config) {
				store::append_history(&config.history_path, &path).await.ok();
			}
		}
		Commands::Cdf { regex, number, history_depth } => {
			let history = store::load_history(&config.history_path)
				.await
				.unwrap_or_default();
			let recent = tail(&history, history_depth);

			let mut counts: HashMap<PathBuf, usize> = HashMap::new();
			for path in recent {
				*counts.entry(path.clone()).or_insert(0) += 1;
			}
			let mut by_count: Vec<(PathBuf, usize)> = counts.into_iter().collect();
			by_count.sort_by(|a, b| b.1.cmp(&a.1));
			let paths: Vec<PathBuf> = by_count.into_iter().map(|(p, _)| p).collect();
			let filtered = filter_paths(paths, &regex, false);
			let limited: Vec<PathBuf> = filtered.into_iter().take(number).collect();

			if let Some(path) = pick_and_print(&limited, "frequent", &config) {
				store::append_history(&config.history_path, &path).await.ok();
			}
		}
		Commands::Cdp { number } => {
			let cwd = std::fs::canonicalize(".")?;
			let history = store::load_history(&config.history_path)
				.await
				.unwrap_or_default();
			let coaccess = CoAccessGraph::build(&history, config.coaccess_window);

			let paths: Vec<PathBuf> = coaccess.neighbors_of(&cwd)
				.iter()
				.take(number)
				.map(|edge| edge.neighbor.clone())
				.collect();

			if let Some(path) = pick_and_print(&paths, "co-accessed (npmi)", &config) {
				store::append_history(&config.history_path, &path).await.ok();
			}
		}
	}

	Ok(())
}
