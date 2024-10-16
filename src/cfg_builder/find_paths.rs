use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use quote::quote;
use std::fs::File;
use std::io::Write;
use crate::cfg_builder::{builder::CfgBuilder, node::CfgNode, node::ConditionalExpr};
use crate::cfg_builder::handle_condition::*;
use petgraph::visit::EdgeRef;
use std::path::{Path};

impl CfgBuilder {

    pub fn generate_simple_paths(&mut self) -> Vec<Vec<NodeIndex>> {
        let condition_nodes = self.get_condition_nodes();
        let mut paths = Vec::new();

        for start_node in condition_nodes {
            let mut visited = HashSet::new();
            self.find_paths(start_node, &mut Vec::new(), &mut paths, &mut visited);
        }

        paths
    }

    fn find_paths(
        &mut self,
        current_node: NodeIndex,
        current_path: &mut Vec<NodeIndex>,
        paths: &mut Vec<Vec<NodeIndex>>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        if visited.contains(&current_node) {
            //return; // Avoid cycles
        }
        visited.insert(current_node);
        current_path.push(current_node);

        // Collect edge information first to avoid borrowing issues
        let edges_info: Vec<(NodeIndex, String)> = self.graph.edges(current_node)
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
            for (target, edge_label) in edges_info {
                if let Some(CfgNode::Condition(_, Some(expr))) = self.graph.node_weight(current_node) {
                    // Clone the condition before modification
                    let cloned_expr = expr.clone();
                    let updated_expr = if edge_label == "false" {
                        // Negate the cloned condition if the edge label is "false"
                        match cloned_expr {
                            ConditionalExpr::If(expr_if) => ConditionalExpr::If(Box::new(CfgBuilder::negate_condition(*expr_if))),
                            ConditionalExpr::While(expr_while) => ConditionalExpr::While(Box::new(CfgBuilder::negate_condition(*expr_while))),
                            _ => cloned_expr,
                        }
                    } else {
                        // Use the cloned condition directly if the edge label is "true"
                        cloned_expr
                    };

                    let condition_str = quote! { #updated_expr }.to_string();
                    let new_label = format!("assume: {}", condition_str);
                    // Update the current path's node with the new label and potentially negated condition
                    *self.graph.node_weight_mut(current_node).unwrap() = CfgNode::Condition(new_label, Some(updated_expr));
                }
                self.find_paths(target, current_path, paths, visited);
            }
        }

        current_path.pop();
        visited.remove(&current_node);
    }

    fn get_condition_nodes(&self) -> Vec<NodeIndex> {
        self.graph.node_indices()
            .filter(|&n| matches!(
                self.graph[n],
                CfgNode::Precondition(_, _)
                | CfgNode::Postcondition(_, _)
                | CfgNode::Invariant(_, _)
                | CfgNode::Cutoff(_)
            ))
            .collect()
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
                        dot_string.push_str(&format!("{} -> {} [label=\"{}\"];\n", from.index(), to.index(), label));
                    } else {
                        dot_string.push_str(&format!("{} -> {};\n", from.index(), to.index()));
                    }
                }
            }

            dot_string.push_str("}\n");

            // Construct the full path for the simple path DOT file inside the base directory
            let dot_file_path = base_path.join(format!("simple_path_{}.dot", i));

            // Create and write to the DOT file
            let mut dot_file = File::create(&dot_file_path).expect("Unable to create DOT file");
            dot_file.write_all(dot_string.as_bytes()).expect("Unable to write to DOT file");
        }
    }
}
