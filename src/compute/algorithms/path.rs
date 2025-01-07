use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use async_trait::async_trait;
use serde_json::json;

use super::{
    AnalysisAlgorithm, Graph, NodeId, Weight, AnalysisResult,
    PathParams, PathType, PathWeightFunction,
};
use crate::error::{Error, Result};

pub struct PathAnalysis {
    algorithm: PathType,
    params: PathParams,
}

#[derive(Debug, Clone)]
struct Path {
    nodes: Vec<NodeId>,
    total_weight: Weight,
}

#[derive(Copy, Clone)]
struct State {
    node: NodeId,
    cost: Weight,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.partial_cmp(&other.cost)
            .unwrap_or(Ordering::Equal)
            .reverse()
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && 
        self.cost.to_bits() == other.cost.to_bits()
    }
}

impl Eq for State {}

impl PathAnalysis {
    pub fn new(algorithm: PathType, params: PathParams) -> Self {
        Self { algorithm, params }
    }

    fn find_shortest_path(
        &self,
        graph: &Graph,
        start: &NodeId,
        end: &NodeId,
    ) -> Result<Option<Path>> {
        let mut distances: HashMap<NodeId, Weight> = HashMap::new();
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();

        distances.insert(*start, 0.0);
        heap.push(State { node: *start, cost: 0.0 });

        while let Some(State { node, cost }) = heap.pop() {
            if node == *end {
                // Reconstruct path
                let mut path = vec![*end];
                let mut current = *end;
                while let Some(&prev) = previous.get(&current) {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                return Ok(Some(Path {
                    nodes: path,
                    total_weight: cost,
                }));
            }

            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);

            if let Some(edges) = graph.get(&node) {
                for &(next, weight) in edges {
                    let adjusted_weight = match self.params.weight_function {
                        PathWeightFunction::Shortest => weight,
                        PathWeightFunction::Longest => -weight,
                        PathWeightFunction::Average => weight,
                        PathWeightFunction::Custom(factor) => weight * factor,
                    };

                    let new_cost = cost + adjusted_weight;
                    let is_better = distances
                        .get(&next)
                        .map_or(true, |&current| new_cost < current);

                    if is_better {
                        distances.insert(next, new_cost);
                        previous.insert(next, node);
                        heap.push(State {
                            node: next,
                            cost: new_cost,
                        });
                    }
                }
            }
        }

        Ok(None)
    }

    fn find_all_paths(
        &self,
        graph: &Graph,
        start: &NodeId,
        end: &NodeId,
    ) -> Result<Vec<Path>> {
        let mut all_paths = Vec::new();
        let mut current_path = vec![*start];
        let mut visited = HashSet::new();
        visited.insert(*start);

        self.dfs_paths(
            graph,
            start,
            end,
            &mut current_path,
            &mut visited,
            &mut all_paths,
            0.0,
        )?;

        Ok(all_paths)
    }

    fn dfs_paths(
        &self,
        graph: &Graph,
        current: &NodeId,
        end: &NodeId,
        path: &mut Vec<NodeId>,
        visited: &mut HashSet<NodeId>,
        all_paths: &mut Vec<Path>,
        weight_so_far: Weight,
    ) -> Result<()> {
        if current == end {
            all_paths.push(Path {
                nodes: path.clone(),
                total_weight: weight_so_far,
            });
            return Ok(());
        }

        if let Some(max_length) = self.params.max_path_length {
            if path.len() >= max_length {
                return Ok(());
            }
        }

        if let Some(edges) = graph.get(current) {
            for &(next, weight) in edges {
                if !visited.contains(&next) {
                    visited.insert(next);
                    path.push(next);

                    let adjusted_weight = match self.params.weight_function {
                        PathWeightFunction::Shortest => weight,
                        PathWeightFunction::Longest => -weight,
                        PathWeightFunction::Average => weight,
                        PathWeightFunction::Custom(factor) => weight * factor,
                    };

                    self.dfs_paths(
                        graph,
                        &next,
                        end,
                        path,
                        visited,
                        all_paths,
                        weight_so_far + adjusted_weight,
                    )?;

                    path.pop();
                    visited.remove(&next);
                }
            }
        }

        Ok(())
    }

    fn find_critical_path(
        &self,
        graph: &Graph,
        start: &NodeId,
        end: &NodeId,
    ) -> Result<Option<Path>> {
        // Find the path with maximum total weight
        let params = PathParams {
            max_path_length: self.params.max_path_length,
            weight_function: PathWeightFunction::Longest,
        };
        let analyzer = PathAnalysis::new(PathType::ShortestPath, params);
        analyzer.find_shortest_path(graph, start, end)
    }

    fn convert_to_analysis_result(&self, paths: Vec<Path>) -> AnalysisResult {
        let mut result = HashMap::new();
        
        // Convert paths to serializable format
        let path_data: Vec<_> = paths.into_iter()
            .map(|path| {
                json!({
                    "nodes": path.nodes.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
                    "weight": path.total_weight,
                })
            })
            .collect();

        result.insert("paths".to_string(), json!(path_data));

        // Store metadata
        result.insert(
            "algorithm".to_string(),
            json!(format!("{:?}", self.algorithm)),
        );
        result.insert(
            "weight_function".to_string(),
            json!(format!("{:?}", self.params.weight_function)),
        );
        if let Some(max_length) = self.params.max_path_length {
            result.insert(
                "max_path_length".to_string(),
                json!(max_length),
            );
        }

        result
    }
}

#[async_trait]
impl AnalysisAlgorithm for PathAnalysis {
    type Input = (Graph, NodeId, NodeId);
    type Parameters = PathParams;

    async fn execute(&self, input: Self::Input) -> Result<AnalysisResult> {
        let (graph, start, end) = input;
        let paths = match self.algorithm {
            PathType::ShortestPath => {
                self.find_shortest_path(&graph, &start, &end)?
                    .map(|p| vec![p])
                    .unwrap_or_default()
            }
            PathType::AllPaths => {
                self.find_all_paths(&graph, &start, &end)?
            }
            PathType::CriticalPath => {
                self.find_critical_path(&graph, &start, &end)?
                    .map(|p| vec![p])
                    .unwrap_or_default()
            }
        };

        Ok(self.convert_to_analysis_result(paths))
    }
} 