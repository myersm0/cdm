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
		#[arg(short, long, default_value_t = 3)]
		depth: usize,
		#[arg(short, long, default_value_t = 15)]
		number: usize,
		#[arg(short, long)]
		regex: Option<String>,
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
		#[arg(short = 'c', long = "current", short_alias = 'p', alias = "prefix")]
		current: bool,
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

fn list_directories(root: &PathBuf, max_depth: usize) -> Vec<PathBuf> {
	let mut result = Vec::new();
	let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
	queue.push_back((root.clone(), 0));

	while let Some((dir, depth)) = queue.pop_front() {
		if depth >= max_depth {
			continue;
		}
		let reader = match std::fs::read_dir(&dir) {
			Ok(r) => r,
			Err(_) => continue,
		};
		let mut children = Vec::new();
		for entry in reader.flatten() {
			let path = entry.path();
			if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
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
	let width = picker::terminal_width();
	PickerConfig {
		title: title.to_string(),
		home_dir: dirs::home_dir(),
		path_aliases: config.path_aliases(),
		max_display_width: width.saturating_sub(6).max(20),
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

fn compile_pattern(pattern: &Option<String>) -> Option<regex::Regex> {
	pattern.as_ref().map(|text| match regex::Regex::new(text) {
		Ok(compiled) => compiled,
		Err(error) => {
			eprintln!("invalid pattern: {}", error);
			std::process::exit(2);
		}
	})
}

fn filter_paths(paths: Vec<PathBuf>, regex: &Option<regex::Regex>, current_directory_only: bool) -> Vec<PathBuf> {
	let cwd = std::env::current_dir().ok();

	paths
		.into_iter()
		.filter(|path| {
			if let Some(ref re) = regex {
				if !re.is_match(&path.to_string_lossy()) {
					return false;
				}
			}
			if current_directory_only {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();
	let config = AppConfig::load();

	match cli.command {
		Commands::Goahead { depth, number, regex } => {
			let cwd = std::fs::canonicalize(".")?;
			let compiled_regex = compile_pattern(&regex);
			let dirs = list_directories(&cwd, depth);
			let filtered = filter_paths(dirs, &compiled_regex, false);
			let limited: Vec<PathBuf> = filtered.into_iter().take(number).collect();
			if let Some(path) = pick_and_print(&limited, "goahead", &config) {
				store::append_history(&config.history_path, &path).ok();
			}
		}
		Commands::Cdr { regex, number, current, history_depth } => {
			let history = store::load_history(&config.history_path)
				.unwrap_or_default();
			let recent = tail(&history, history_depth);

			let mut seen = HashSet::new();
			let paths: Vec<PathBuf> = recent.iter()
				.rev()
				.filter(|p| seen.insert((*p).clone()))
				.cloned()
				.collect();
			let filtered = filter_paths(paths, &compile_pattern(&regex), current);
			let mut limited: Vec<PathBuf> = filtered.into_iter().take(number).collect();
			limited.reverse();

			if let Some(path) = pick_and_print(&limited, "recent", &config) {
				store::append_history(&config.history_path, &path).ok();
			}
		}
		Commands::Cdf { regex, number, history_depth } => {
			let history = store::load_history(&config.history_path)
				.unwrap_or_default();
			let recent = tail(&history, history_depth);

			let mut counts: HashMap<PathBuf, usize> = HashMap::new();
			for path in recent {
				*counts.entry(path.clone()).or_insert(0) += 1;
			}
			let mut by_count: Vec<(PathBuf, usize)> = counts.into_iter().collect();
			by_count.sort_by(|a, b| b.1.cmp(&a.1));
			let paths: Vec<PathBuf> = by_count.into_iter().map(|(p, _)| p).collect();
			let filtered = filter_paths(paths, &compile_pattern(&regex), false);
			let mut limited: Vec<PathBuf> = filtered.into_iter().take(number).collect();
			limited.reverse();

			if let Some(path) = pick_and_print(&limited, "frequent", &config) {
				store::append_history(&config.history_path, &path).ok();
			}
		}
		Commands::Cdp { number } => {
			let cwd = std::fs::canonicalize(".")?;
			let history = store::load_history(&config.history_path)
				.unwrap_or_default();
			let coaccess = CoAccessGraph::build(&history, config.coaccess_window);

			let mut paths: Vec<PathBuf> = coaccess.neighbors_of(&cwd)
				.iter()
				.take(number)
				.map(|edge| edge.neighbor.clone())
				.collect();
			paths.reverse();

			if let Some(path) = pick_and_print(&paths, "co-accessed (npmi)", &config) {
				store::append_history(&config.history_path, &path).ok();
			}
		}
	}

	Ok(())
}
