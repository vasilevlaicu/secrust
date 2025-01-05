use syn::ExprIf;

use crate::cfg_builder::builder::CfgBuilder;
use crate::cfg_builder::node::{CfgNode, ConditionalExpr};
use proc_macro2::Span;
use quote::quote;
use syn::{token, visit::Visit, Expr, ExprParen, ExprUnary, Pat, UnOp};

impl CfgBuilder {
    pub fn handle_if_statement(&mut self, expr_if: &ExprIf) {
        let cond_str = self.format_condition(&expr_if.cond);
        let cond_label = if self.next_edge_label == Some("false".to_string()) {
            format!("else if: {}", cond_str)
        } else {
            format!("if: {}", cond_str)
        };
        let cond_expr = ConditionalExpr::If(expr_if.cond.clone());
        let cond_node = self.add_node(CfgNode::new_condition(cond_label, cond_expr));

        // Processing the true branch
        self.next_edge_label = Some("true".to_string());
        self.current_node = Some(cond_node.clone());
        self.visit_block(&expr_if.then_branch);
        let true_branch_end = self.current_node;

        // Create a merge point node
        let merge_node = self.add_node_without_edge(CfgNode::MergePoint);

        // Connect the true branch end to the merge point
        if let Some(true_end) = true_branch_end {
            self.add_edge_with_label(true_end, merge_node, "".to_string());
        }

        // Handling the else branch if present
        if let Some((_, else_branch)) = &expr_if.else_branch {
            self.current_node = Some(cond_node.clone());
            self.next_edge_label = Some("false".to_string());
            match &**else_branch {
                Expr::If(elseif) => {
                    // Handle else if with recursion
                    self.handle_if_statement(elseif);
                }
                Expr::Block(block) => {
                    self.visit_block(&block.block);
                }
                _ => {
                    self.visit_expr(else_branch);
                }
            }

            // Connect the end of the else branch to the merge point
            if let Some(false_end) = self.current_node {
                self.add_edge_with_label(false_end, merge_node, "".to_string());
            }
        } else {
            // If there is no else branch, connect the condition node to the merge point with a 'false' label
            self.add_edge_with_label(cond_node, merge_node, "false".to_string());
        }

        // Continue from the merge point after if-else
        self.current_node = Some(merge_node);
    }
    pub fn format_pattern_condition(&self, pat: &Pat) -> String {
        let raw_string = quote!(#pat).to_string();
        Self::clean_up_formatting(&raw_string)
    }
    pub fn negate_condition(expr: Expr) -> Expr {
        // unary negation expression with '!'
        let paren_expr = ExprParen {
            attrs: Vec::new(),
            paren_token: token::Paren(Span::call_site()),
            expr: Box::new(expr),
        };

        // create a unary negation expression with '!' applied to the parenthesized expression
        let not_expr = ExprUnary {
            attrs: Vec::new(),
            op: UnOp::Not(token::Bang {
                spans: [Span::call_site()],
            }),
            expr: Box::new(Expr::Paren(paren_expr)),
        };

        Expr::Unary(not_expr)
    }
}
