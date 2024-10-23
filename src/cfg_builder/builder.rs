use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use quote::quote;
use syn::{visit::{self, Visit}, Expr, ExprAssign, ExprReturn, Block, File as SynFile, ItemFn, Pat, Stmt};

use crate::cfg_builder::node::{CfgNode, ConditionalExpr};
use crate::cfg_builder::handle_condition::*;
use crate::cfg_builder::handle_loops::*;
use crate::cfg_builder::handle_macros::*;
use crate::cfg_builder::handle_return::*;
use crate::cfg_builder::handle_call::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExternalMethod {
    pub name: String,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalMethods {
    pub external_methods: Vec<ExternalMethod>,
}

pub struct CfgBuilder {
    pub graph: DiGraph<CfgNode, String>,
    pub current_node: Option<NodeIndex>,
    pub next_edge_label: Option<String>,
    pub external_conditions: ExternalMethods,
    pub postconditions: Vec<CfgNode>,
}

impl CfgBuilder {
    pub fn new() -> Self {
        // Should check for external config in user's project directory
        let external_conditions = match Self::parse_external_definitions("src/config/conditions.json") {
            Ok(conditions) => conditions,
            Err(e) => {
                eprintln!("Failed to load external conditions: {}", e);
                ExternalMethods { external_methods: vec![] }
            }
        };

        CfgBuilder {
            graph: DiGraph::new(),
            current_node: None,
            next_edge_label: None,
            external_conditions,
            postconditions: Vec::new(),
        }
    }

    pub fn parse_external_definitions(file_path: &str) -> Result<ExternalMethods, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file_path).exists() {
            eprintln!("Warning: External conditions file not found. Using empty conditions.");
            return Ok(ExternalMethods { external_methods: vec![] });
        }
    
        let file_content = fs::read_to_string(file_path)?;
        let external_methods: ExternalMethods = serde_json::from_str(&file_content)?;
        Ok(external_methods)
    }
    

    pub fn add_postconditions(&mut self) {
        let postconditions = self.postconditions.clone();
        for postcondition in postconditions {
            self.add_node(postcondition);
        }
        self.postconditions.clear();
    }

    pub fn add_node(&mut self, node: CfgNode) -> NodeIndex {
        let index = self.graph.add_node(node);
        if let Some(current) = self.current_node {
            // Use the label for the next edge if available
            let label = self.next_edge_label.clone().unwrap_or_else(|| "".to_string());
            self.graph.add_edge(current, index, label);
            // Reset the edge label
            self.next_edge_label = None;
        }
        self.current_node = Some(index);
        index
    }

    pub fn add_node_without_edge(&mut self, node: CfgNode) -> NodeIndex {
        let index = self.graph.add_node(node);
        self.current_node = Some(index);
        index
    }

    pub fn add_edge_with_label(&mut self, from: NodeIndex, to: NodeIndex, label: String) {
        self.graph.add_edge(from, to, label);
    }

    pub fn to_dot(&self) -> String {
        let mut dot_string = String::new();
        dot_string.push_str("digraph G {\n");
        for node in self.graph.node_indices() {
            let cfg_node = &self.graph[node];
            dot_string.push_str(&cfg_node.format_dot(node.index()));
            dot_string.push('\n');
        }
        for edge in self.graph.edge_references() {
            let source = edge.source().index();
            let target = edge.target().index();
            let label = edge.weight();
            dot_string.push_str(&format!("{} -> {} [label=\"{}\"];\n", source, target, label));
        }
        dot_string.push_str("}\n");
        dot_string
    }

    pub fn clean_up_formatting(input: &str) -> String {
        let re = Regex::new(r"\s*([\(\)\[\]!\.,;])\s*").unwrap();
        let cleaned = re.replace_all(input, "$1").to_string();

        cleaned.replace("vec! [", "vec![")
               .replace("+ ", " + ")
    }

    pub fn format_condition(&self, expr: &Box<Expr>) -> String {
        let raw_string = quote!(#expr).to_string();
        Self::clean_up_formatting(&raw_string)
    }

    pub fn post_process(&mut self) {
        let mut merge_nodes_to_process: Vec<NodeIndex> = self.graph.node_indices()
            .filter(|&n| matches!(self.graph[n], CfgNode::MergePoint))
            .collect();

        while let Some(merge_node) = merge_nodes_to_process.pop() {
            // Check if the merge node has edges (i.e., is still part of the graph)
            if self.graph.edges(merge_node).count() == 0 {
                continue;
            }

            // Find outgoing edges of the merge node
            let edges: Vec<_> = self.graph.edges(merge_node).collect();

            if edges.len() == 1 {
                let target = edges[0].target();
                if matches!(self.graph[target], CfgNode::MergePoint) {
                    // If the target is another merge node, merge them
                    self.merge_merge_nodes(merge_node, target);
                    merge_nodes_to_process.push(target);
                } else {
                    // If the target is not a merge node, redirect incoming edges and remove the merge node
                    self.redirect_edges_and_remove(merge_node, target);
                }
            }
        }
        // Clean up formatting in the node labels
        for node in self.graph.node_indices() {
            if let CfgNode::Condition(label, _) | CfgNode::Statement(label, _) = &mut self.graph[node] {
                *label = CfgBuilder::clean_up_formatting(label);
            }
        }
    }

    fn merge_merge_nodes(&mut self, source: NodeIndex, target: NodeIndex) {
        let incoming_edges: Vec<_> = self.graph.edges_directed(source, petgraph::Direction::Incoming)
            .map(|e| (e.source(), e.weight().clone()))
            .collect();
    
        for (source_of_edge, weight) in incoming_edges {
            self.graph.add_edge(source_of_edge, target, weight);
        }
        self.graph.remove_node(source);
    }
    
    fn redirect_edges_and_remove(&mut self, source: NodeIndex, new_target: NodeIndex) {
        let incoming_edges: Vec<_> = self.graph.edges_directed(source, petgraph::Direction::Incoming)
            .map(|e| (e.source(), e.weight().clone()))
            .collect();
    
        for (source_of_edge, weight) in incoming_edges {
            self.graph.add_edge(source_of_edge, new_target, weight);
        }
    
        self.graph.remove_node(source);
    }

    fn format_macro_args(&self, tokens: &proc_macro2::TokenStream) -> String {
        let tokens_str = tokens.to_string();
        tokens_str.trim_start_matches("!(")
                  .trim_end_matches(')')
                  .trim_matches(|c| c == '"' || c == '\'')
                  .to_string()
    }
}

impl Visit<'_> for CfgBuilder {
    fn visit_file(&mut self, i: &SynFile) {
        visit::visit_file(self, i);
    }

    fn visit_item_fn(&mut self, i: &ItemFn) {
        let func_name = i.sig.ident.to_string();
        let func_node = self.add_node(CfgNode::new_function(func_name.clone(), i.clone()));

        self.current_node = Some(func_node);

        for stmt in &i.block.stmts {
            match stmt {
                Stmt::Semi(expr, _) => {
                    if let Expr::Macro(expr_macro) = expr {
                        if let Some(macro_ident) = expr_macro.mac.path.get_ident() {
                            let macro_name = macro_ident.to_string();
                            let macro_args = self.format_macro_args(&expr_macro.mac.tokens);
                            let node = match macro_name.as_str() {
                                "pre" => CfgNode::new_precondition(macro_args.clone(), Expr::Macro(expr_macro.clone())),
                                "post" => {
                                    let post_node = CfgNode::new_postcondition(macro_args.clone(), Expr::Macro(expr_macro.clone()));
                                    self.postconditions.push(post_node.clone());
                                    post_node
                                },
                                "invariant" => CfgNode::new_invariant(macro_args.clone(), Expr::Macro(expr_macro.clone())),
                                _ => {
                                    let expr_str = quote!(#i).to_string();
                                    CfgNode::new_statement(expr_str, Stmt::Expr(Expr::Macro(expr_macro.clone())))
                                }
                            };
                            if macro_name.as_str() != "post" {
                                self.add_node(node);
                            }
                        } else {
                            self.visit_expr(expr);
                        }
                    } else {
                        self.visit_expr(expr);
                    }
                },
                _ => self.visit_stmt(stmt),
            }
        }
        self.add_postconditions();

        self.current_node = None;
    }

    fn visit_expr(&mut self, i: &Expr) {
        // println!("Visiting expression: {:?}", i);
        match i {
            Expr::If(expr_if) => self.handle_if_statement(expr_if),
            Expr::While(expr_while) => self.handle_while_loop(expr_while),
            Expr::ForLoop(expr_for) => self.handle_for_loop(expr_for),
            Expr::Return(expr_return) => {
                // println!("Handling return statement: {:?}", expr_return);
                self.handle_return_statement(expr_return);
            },
            Expr::Call(expr_call) => self.handle_call(expr_call),
            Expr::MethodCall(expr_method_call) => self.handle_method_call(expr_method_call),
            Expr::Macro(expr_macro) => {
                self.process_macro(expr_macro);
            },
            Expr::Array(expr_array) => {
                for elem in &expr_array.elems {
                    self.visit_expr(elem); // Recursively visit to catch nested macros
                }
            },
            _ => {
                // Handling invariant macro 
                if let Expr::Macro(expr_macro) = i {
                    if let Some(macro_ident) = expr_macro.mac.path.get_ident() {
                        if macro_ident == "invariant" {
                            // Handling invariant
                            let invariant_str = self.format_macro_args(&expr_macro.mac.tokens);
                            self.add_node(CfgNode::new_invariant(invariant_str, Expr::Macro(expr_macro.clone())));
                            return;
                        }
                    }
                }
                // else a simple expression.
                let expr_str = quote!(#i).to_string();
                let call_statement = Stmt::Expr(i.clone());
                self.add_node(CfgNode::new_statement(expr_str, call_statement));
            },
        }
    }
    fn visit_block(&mut self, i: &Block) {
        for stmt in &i.stmts {
            self.visit_stmt(stmt);
        }
    }
    fn visit_stmt(&mut self, i: &Stmt) {
        match i {
            Stmt::Local(local) => {
                // Handle local variable declarations
                let local_str = format!("{}", quote!(#local));
                self.add_node(CfgNode::new_statement(local_str, Stmt::Local(local.clone())));
                
            }
            Stmt::Expr(expr) | Stmt::Semi(expr, _) => self.visit_expr(expr),
            _ => visit::visit_stmt(self, i),
        }
    }
}
