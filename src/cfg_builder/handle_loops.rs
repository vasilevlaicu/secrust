use syn::{visit::{self, Visit}, ExprForLoop, ExprWhile};

use crate::cfg_builder::builder::CfgBuilder;
use crate::cfg_builder::node::{CfgNode, ConditionalExpr};

impl CfgBuilder {
    pub fn handle_for_loop(&mut self, expr_for: &syn::ExprForLoop) {
        // Check if the last node was an invariant
        let invariant_node = self.current_node
            .filter(|&current| matches!(self.graph[current], CfgNode::Invariant(_, _)));
    
        let loop_back_node;
    
        if invariant_node.is_none() {
            // Add the "@Cutoff" node if no invariant is present
            let cutoff_node = self.add_node(CfgNode::new_cutoff("".to_string()));
            loop_back_node = cutoff_node;
        } else {
            loop_back_node = invariant_node.unwrap();
        }
    
        let loop_var = self.format_pattern_condition(&expr_for.pat);
        let iterator = self.format_condition(&expr_for.expr);
        let cond_label = format!("for {} in {}", loop_var, iterator);
        let cond_expr = ConditionalExpr::ForLoop(expr_for.clone());
        let cond_node = self.add_node(CfgNode::new_condition(cond_label, cond_expr));
    
        // Process the loop body
        self.current_node = Some(cond_node);
        self.next_edge_label = Some("true".to_string());
        self.visit_block(&expr_for.body);
    
        // Link back to the loop_back_node after the loop body
        if let Some(end_node) = self.current_node {
            self.add_edge_with_label(end_node, loop_back_node, "back to loop".to_string());
        }
    
        // Create a merge node for the exit of the loop
        let merge_node = self.add_node_without_edge(CfgNode::MergePoint);
        self.add_edge_with_label(cond_node, merge_node, "false".to_string());
    
        // Continue from the merge point after the loop
        self.current_node = Some(merge_node);
    }

    pub fn handle_while_loop(&mut self, expr_while: &ExprWhile) {
        // Check if the last node was an invariant
        let invariant_node = self.current_node
            .filter(|&current| matches!(self.graph[current], CfgNode::Invariant(_, _)));

        let loop_back_node;

        if invariant_node.is_none() {
            // Add the "@Cutoff" node if no invariant is present
            let cutoff_node = self.add_node(CfgNode::new_cutoff("".to_string()));
            loop_back_node = cutoff_node;
        } else {
            loop_back_node = invariant_node.unwrap();
        }

        // Add the "while" condition node
        let cond_str = self.format_condition(&expr_while.cond);
        let cond_expr = ConditionalExpr::While(expr_while.cond.clone());
        let cond_node = self.add_node(CfgNode::new_condition(format!("while: {}", cond_str), cond_expr));

        // Process the loop body
        self.current_node = Some(cond_node);
        self.next_edge_label = Some("true".to_string());
        self.visit_block(&expr_while.body);

        // Link back to the loop_back_node after the loop body
        if let Some(end_node) = self.current_node {
            self.add_edge_with_label(end_node, loop_back_node, "back to loop".to_string());
        }

        // Create a merge node for the false branch of the condition
        let merge_node = self.add_node_without_edge(CfgNode::MergePoint);
        self.add_edge_with_label(cond_node, merge_node, "false".to_string());

        // Continue from the merge point after the loop
        self.current_node = Some(merge_node);
    }
}
