use crate::cfg_builder::{builder::CfgBuilder, node::CfgNode};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::fs::File;
use std::io::Write;
use std::path::Path;

impl CfgBuilder {
    pub fn generate_basic_paths(&mut self) -> Vec<Vec<NodeIndex>> {
        let condition_nodes = self.get_condition_nodes();
        let mut paths = Vec::new();

        for start_node in condition_nodes {
            self.find_paths(start_node, &mut Vec::new(), &mut paths);
        }

        // Process paths to check for loops and invariants
        for path in paths.iter_mut() {
            if self.is_loop_path(path) {
                self.process_loop_invariant_path(path);
            }
        }

        paths
    }

    fn get_condition_nodes(&self) -> Vec<NodeIndex> {
        self.graph
            .node_indices()
            .filter(|&n| {
                matches!(
                    self.graph[n],
                    CfgNode::Precondition(_, _)
                        | CfgNode::Postcondition(_, _)
                        | CfgNode::Invariant(_, _)
                        | CfgNode::Cutoff(_)
                )
            })
            .collect()
    }

    fn find_paths(
        &mut self,
        current_node: NodeIndex,
        current_path: &mut Vec<NodeIndex>,
        paths: &mut Vec<Vec<NodeIndex>>,
    ) {
        current_path.push(current_node);

        // Collect edge information first to avoid borrowing issues
        let edges_info: Vec<(NodeIndex, String)> = self
            .graph
            .edges(current_node)
            .map(|edge| (edge.target(), edge.weight().clone()))
            .collect();

        // Check for a terminal condition or another condition node
        if matches!(
            self.graph[current_node],
            CfgNode::Precondition(_, _)
                | CfgNode::Postcondition(_, _)
                | CfgNode::Invariant(_, _)
                | CfgNode::Cutoff(_)
        ) && current_path.len() > 1
        {
            paths.push(current_path.clone());
        } else {
            // Continue exploring adjacent nodes
            for (target, _edge_label) in edges_info {
                self.find_paths(target, current_path, paths);
            }
        }

        current_path.pop();
    }

    fn is_loop_path(&self, path: &Vec<NodeIndex>) -> bool {
        // Check if there's a "back to loop" edge in the path, indicating a loop structure
        path.windows(2).any(|pair| {
            if let [from, to] = pair {
                self.graph
                    .edges_connecting(*from, *to)
                    .any(|edge| edge.weight() == "back to loop")
            } else {
                false
            }
        })
    }

    fn process_loop_invariant_path(&mut self, path: &mut Vec<NodeIndex>) {
        if let Some(&first_node) = path.first() {
            if let CfgNode::Invariant(cond, expr) = &self.graph[first_node] {
                // Create a new terminal node with the same invariant condition
                let new_terminal_node = self
                    .graph
                    .add_node(CfgNode::Invariant(cond.clone(), expr.clone()));

                // Remove the last node in the path
                path.pop();

                // Add the new terminal node
                path.push(new_terminal_node);
            }
        }
    }

    pub fn write_paths_to_dot_files(&self, paths: Vec<Vec<NodeIndex>>, base_path: &Path) {
        // Create the output directory if it doesn't exist
        std::fs::create_dir_all(base_path).expect("Unable to create base directory for paths");

        for (i, path) in paths.iter().enumerate() {
            let mut dot_string = String::from("digraph Path {\n");

            // Add nodes to the DOT string
            for &node in path {
                let cfg_node = &self.graph[node];
                dot_string.push_str(&cfg_node.format_dot(node.index()));
                dot_string.push('\n');
            }

            // Add edges for path
            for window in path.windows(2) {
                if let [from, to] = window {
                    // Find all edges connecting 'from' to 'to'
                    let edges: Vec<_> = self.graph.edges_connecting(*from, *to).collect();

                    if let Some(edge) = edges.first() {
                        let label = &self.graph[edge.id()];
                        dot_string.push_str(&format!(
                            "{} -> {} [label=\"{}\"];\n",
                            from.index(),
                            to.index(),
                            label
                        ));
                    } else {
                        dot_string.push_str(&format!("{} -> {};\n", from.index(), to.index()));
                    }
                }
            }

            dot_string.push_str("}\n");

            // Write the DOT file
            let dot_file_path = base_path.join(format!("basic_path_{}.dot", i));
            let mut dot_file = File::create(&dot_file_path).expect("Unable to create DOT file");
            dot_file
                .write_all(dot_string.as_bytes())
                .expect("Unable to write to DOT file");
        }
    }
}
