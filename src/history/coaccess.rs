use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CoAccessEdge {
	pub neighbor: PathBuf,
	pub score: f64,
}

#[derive(Debug, Clone)]
pub struct CoAccessGraph {
	pub edges: HashMap<PathBuf, Vec<CoAccessEdge>>,
	pub window_size: usize,
}

impl CoAccessGraph {
	pub fn build(history: &[PathBuf], window_size: usize) -> Self {
		let mut edges: HashMap<PathBuf, Vec<CoAccessEdge>> = HashMap::new();

		if history.len() < window_size || window_size < 2 {
			return Self { edges, window_size };
		}

		let total_windows = history.len() - window_size + 1;
		let total_f = total_windows as f64;

		let mut marginal_counts: HashMap<PathBuf, u32> = HashMap::new();
		let mut joint_counts: HashMap<(PathBuf, PathBuf), u32> = HashMap::new();

		for start in 0..total_windows {
			let window: HashSet<&PathBuf> = history[start..start + window_size].iter().collect();
			let unique: Vec<&PathBuf> = window.into_iter().collect();

			for path in &unique {
				*marginal_counts.entry((*path).clone()).or_insert(0) += 1;
			}

			for i in 0..unique.len() {
				for j in (i + 1)..unique.len() {
					let key = if unique[i] < unique[j] {
						(unique[i].clone(), unique[j].clone())
					} else {
						(unique[j].clone(), unique[i].clone())
					};
					*joint_counts.entry(key).or_insert(0) += 1;
				}
			}
		}

		for ((path_a, path_b), joint) in &joint_counts {
			let p_joint = *joint as f64 / total_f;
			let p_a = marginal_counts[path_a] as f64 / total_f;
			let p_b = marginal_counts[path_b] as f64 / total_f;

			let pmi = (p_joint / (p_a * p_b)).ln();
			let npmi = pmi / -p_joint.ln();

			if npmi <= 0.0 {
				continue;
			}

			edges.entry(path_a.clone()).or_default().push(CoAccessEdge {
				neighbor: path_b.clone(),
				score: npmi,
			});
			edges.entry(path_b.clone()).or_default().push(CoAccessEdge {
				neighbor: path_a.clone(),
				score: npmi,
			});
		}

		for neighbors in edges.values_mut() {
			neighbors.sort_by(|a, b| {
				b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
			});
		}

		Self { edges, window_size }
	}

	pub fn neighbors_of(&self, path: &PathBuf) -> &[CoAccessEdge] {
		self.edges.get(path).map(|v| v.as_slice()).unwrap_or(&[])
	}

	pub fn score_for(&self, from: &PathBuf, to: &PathBuf) -> Option<f64> {
		self.neighbors_of(from)
			.iter()
			.find(|e| e.neighbor == *to)
			.map(|e| e.score)
	}
}
