use crate::cfg_builder::node::CfgNode;
use crate::cfg_builder::{builder::CfgBuilder, node::ConditionalExpr};
use petgraph::graph::NodeIndex;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
/// This module handles variable substitution and logical condition chaining for Control Flow Graph (CFG) paths.
/// This is the module to verify to add any new operations we'd like the cargo to handle.
///
/// The primary goal of this module is to substitute variables within the CFG nodes, particularly for conditions
/// like postconditions, and invariants, enabling weakest precondition calculations.
///
/// Key responsibilities include:
/// - Traversing CFG paths and applying wp calculus instructions based on the current variable state.
/// - Handling postcondition substitution to create a single logical condition for verification.
/// - Formatting and chaining conditions using logical implication (`>>`).
/// - Parsing assignment statements and substituting variables within expressions.
/// - Recursively substituting variables in complex Rust syntax structures, including `if` conditions, loops,
///   macro calls, and nested expressions.
/// - Adding parentheses where necessary to ensure proper precedence in logical conditions.
///
/// This module plays a critical role in translating the program's logical flow into a format suitable for formal verification.
///
/// Dependencies:
/// - Relies on the `syn` crate for Rust syntax parsing.
/// - Uses `petgraph` for traversing the CFG and maintaining node relationships.
use syn::{
    Block, Expr, ExprAssign, ExprBinary, ExprBlock, ExprCall, ExprIf, ExprMacro, ExprParen,
    ExprUnary, Local, Macro, Stmt,
};

impl CfgBuilder {
    pub fn apply_wp_calculus(&self, paths: &[Vec<NodeIndex>]) -> Vec<String> {
        let mut updated_postconditions = Vec::new();

        for path in paths {
            let mut variable_state = HashMap::new();
            let mut working_condition: Option<syn::Expr> = None;

            // Traverse the path in reverse (from postcondition up to precondition)
            for &node_index in path.iter().rev() {
                match &self.graph[node_index] {
                    CfgNode::Statement(stmt_str, _stmt_option) => {
                        if let Some((var, expr)) = self.parse_assignment(stmt_str) {
                            // Check if there is a working condition that needs substitution
                            if let Some(mut cond) = working_condition.take() {
                                // Substitute once per variable
                                cond = self.recursive_substitution(&cond, &var, &expr);
                                working_condition = Some(cond);
                            }

                            // Track the current variable state for potential future substitution
                            variable_state.insert(var.clone(), expr.clone());
                            //println!("varState: {:?}", variable_state);
                        }
                    }
                    CfgNode::Condition(_, Some(conditional_expr)) => {
                        // Don't substitute conditions but add them in the implication chain
                        let is_false_branch = self.is_false_branch(&path, node_index);
                        let updated_expr = if is_false_branch {
                            // Negate the condition if we are on the false branch
                            match conditional_expr {
                                ConditionalExpr::If(expr_if) => ConditionalExpr::If(Box::new(
                                    CfgBuilder::negate_condition(*expr_if.clone()),
                                )),
                                ConditionalExpr::While(expr_while) => ConditionalExpr::While(
                                    Box::new(CfgBuilder::negate_condition(*expr_while.clone())),
                                ),
                                _ => conditional_expr.clone(),
                            }
                        } else {
                            match conditional_expr {
                                ConditionalExpr::If(expr_if) => ConditionalExpr::If(Box::new(
                                    Self::wrap_with_parens(*expr_if.clone()),
                                )),
                                ConditionalExpr::While(expr_while) => ConditionalExpr::While(
                                    Box::new(Self::wrap_with_parens(*expr_while.clone())),
                                ),
                                _ => conditional_expr.clone(),
                            }
                        };

                        let expr = updated_expr.to_syn_expr();
                        working_condition =
                            Some(if let Some(existing_cond) = working_condition.take() {
                                syn::parse2(quote! { #expr >> #existing_cond })
                                    .expect("Failed to parse condition implication")
                            } else {
                                expr.clone()
                            });
                    }
                    // TODO check what's extra here
                    CfgNode::Postcondition(_, Some(expr)) | CfgNode::Invariant(_, Some(expr)) => {
                        // Substitute variables in the postcondition/invariant and chain with the current condition
                        let expr = expr.clone();
                        working_condition =
                            Some(if let Some(existing_cond) = working_condition.take() {
                                syn::parse2(quote! { #expr >> #existing_cond })
                                    .expect("Failed to parse conjunction")
                            } else {
                                expr
                            });
                    }
                    CfgNode::Precondition(_, Some(expr)) => {
                        // Chain with the current condition
                        let expr = expr.clone();
                        working_condition =
                            Some(if let Some(existing_cond) = working_condition.take() {
                                syn::parse2(quote! { #expr >> #existing_cond })
                                    .expect("Failed to parse conjunction")
                            } else {
                                expr
                            });
                    }
                    _ => {}
                }
            }

            if let Some(cond) = working_condition {
                updated_postconditions.push(quote! { #cond }.to_string());
            }
        }

        updated_postconditions
    }

    fn is_false_branch(&self, path: &[NodeIndex], current_node: NodeIndex) -> bool {
        // Iterate over edges connecting from the current node in the path
        let current_index = path.iter().position(|&n| n == current_node);
        if let Some(index) = current_index {
            if let Some(next_node) = path.get(index + 1) {
                if let Some(edge) = self.graph.edges_connecting(current_node, *next_node).next() {
                    return edge.weight() == "false";
                }
            }
        }
        false
    }

    fn wrap_with_parens(expr: Expr) -> Expr {
        Expr::Paren(ExprParen {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            expr: Box::new(expr),
        })
    }

    pub fn recursive_substitution(
        &self,
        expr: &Expr,
        var: &str,
        replacement_without_paren: &Expr,
    ) -> Expr {
        // println!("Substituting in expr: {:?}", quote! {#expr});
        //self.print_expr_details(expr);
        let replacement = &Expr::Paren(ExprParen {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            expr: Box::new(replacement_without_paren.clone()),
        });

        match expr {
            Expr::Path(expr_path) => {
                if expr_path.path.is_ident(var) {
                    // println!("Substituting {} with {}", var, quote! {#replacement});
                    replacement.clone()
                } else {
                    expr.clone()
                }
            }
            Expr::Macro(expr_macro) => {
                let new_tokens =
                    self.substitute_in_token_stream(&expr_macro.mac.tokens, var, replacement);
                // println!("new_tokens:{:?}", new_tokens);
                Expr::Macro(ExprMacro {
                    attrs: expr_macro.attrs.clone(),
                    mac: Macro {
                        path: expr_macro.mac.path.clone(),
                        bang_token: expr_macro.mac.bang_token,
                        delimiter: expr_macro.mac.delimiter.clone(),
                        tokens: new_tokens,
                    },
                })
            }
            Expr::Assign(assign) => Expr::Assign(ExprAssign {
                attrs: assign.attrs.clone(),
                left: Box::new(self.recursive_substitution(&assign.left, var, replacement)),
                eq_token: assign.eq_token,
                right: Box::new(self.recursive_substitution(&assign.right, var, replacement)),
            }),
            Expr::Binary(bin) => {
                // println!("Binary Expression: left = {}, op = {}, right = {}", quote! {#bin.left}, quote! {#bin.op}, quote! {#bin.right});
                Expr::Binary(ExprBinary {
                    attrs: bin.attrs.clone(),
                    left: Box::new(self.recursive_substitution(&bin.left, var, replacement)),
                    op: bin.op.clone(),
                    right: Box::new(self.recursive_substitution(&bin.right, var, replacement)),
                })
            }
            Expr::Call(call) => Expr::Call(ExprCall {
                attrs: call.attrs.clone(),
                func: Box::new(self.recursive_substitution(&call.func, var, replacement)),
                paren_token: call.paren_token,
                args: call
                    .args
                    .iter()
                    .map(|arg| self.recursive_substitution(arg, var, replacement))
                    .collect(),
            }),
            Expr::Unary(unary) => Expr::Unary(ExprUnary {
                attrs: unary.attrs.clone(),
                op: unary.op.clone(),
                expr: Box::new(self.recursive_substitution(&unary.expr, var, replacement)),
            }),
            Expr::Paren(paren) => Expr::Paren(ExprParen {
                attrs: paren.attrs.clone(),
                paren_token: paren.paren_token,
                expr: Box::new(self.recursive_substitution(&paren.expr, var, replacement)),
            }),
            Expr::Block(block) => Expr::Block(ExprBlock {
                attrs: block.attrs.clone(),
                label: block.label.clone(),
                block: Block {
                    stmts: block
                        .block
                        .stmts
                        .iter()
                        .map(|stmt| self.recursive_substitute_stmt(stmt, var, replacement))
                        .collect(),
                    ..block.block.clone()
                },
            }),
            Expr::If(expr_if) => Expr::If(ExprIf {
                cond: Box::new(self.recursive_substitution(&expr_if.cond, var, replacement)),
                then_branch: Block {
                    stmts: expr_if
                        .then_branch
                        .stmts
                        .iter()
                        .map(|stmt| self.recursive_substitute_stmt(stmt, var, replacement))
                        .collect(),
                    ..expr_if.then_branch.clone()
                },
                else_branch: expr_if.else_branch.as_ref().map(|else_expr| {
                    (
                        else_expr.0.clone(),
                        Box::new(self.recursive_substitution(&else_expr.1, var, replacement)),
                    )
                }),
                ..expr_if.clone()
            }),
            _ => {
                // println!("No substitution performed for: {}", quote! {#expr});
                expr.clone()
            }
        }
    }

    fn recursive_substitute_stmt(&self, stmt: &Stmt, var: &str, replacement: &Expr) -> Stmt {
        match stmt {
            Stmt::Expr(expr) => Stmt::Expr(self.recursive_substitution(expr, var, replacement)),
            Stmt::Semi(expr, semi) => Stmt::Semi(
                self.recursive_substitution(expr, var, replacement),
                semi.clone(),
            ),
            Stmt::Local(local) => {
                let init = local.init.as_ref().map(|(eq, expr)| {
                    (
                        *eq,
                        Box::new(self.recursive_substitution(expr, var, replacement)),
                    )
                });
                Stmt::Local(Local {
                    pat: local.pat.clone(),
                    init,
                    attrs: local.attrs.clone(),
                    let_token: local.let_token.clone(),
                    semi_token: local.semi_token.clone(),
                })
            }
            _ => stmt.clone(),
        }
    }

    fn parse_assignment(&self, stmt: &str) -> Option<(String, syn::Expr)> {
        // Debug print the input statement
        // Ensure the statement ends with a semicolon
        let stmt = if stmt.trim_end().ends_with(';') {
            stmt.to_string()
        } else {
            format!("{};", stmt.trim_end())
        };

        //println!("Parsing statement: {}", stmt);

        // Parse the statement into a syn::Stmt
        let stmt: syn::Stmt = match syn::parse_str(&stmt) {
            Ok(s) => s,
            Err(_e) => {
                // println!("Failed to parse statement: {}", e);
                return None;
            }
        };

        // Debug print the parsed statement
        //println!("Parsed syn::Stmt: {:#?}", &stmt);

        if let syn::Stmt::Expr(syn::Expr::Assign(assign))
        | syn::Stmt::Semi(syn::Expr::Assign(assign), _) = stmt.clone()
        {
            // Handle simple assignments like 'count = 0;'
            if let syn::Expr::Path(path) = *assign.left {
                if let Some(ident) = path.path.get_ident() {
                    let var = ident.to_string();
                    // println!("Found assignment: {} = {:?}", var, *assign.right);
                    return Some((var, *assign.right));
                }
            }
        } else if let syn::Stmt::Expr(syn::Expr::AssignOp(assign_op))
        | syn::Stmt::Semi(syn::Expr::AssignOp(assign_op), _) = stmt.clone()
        {
            // Handle compound assignments like 'count += 1;'
            if let syn::Expr::Path(path) = *assign_op.left {
                if let Some(ident) = path.path.get_ident() {
                    let var = ident.to_string();
                    let right_expr = syn::Expr::Binary(syn::ExprBinary {
                        attrs: vec![],
                        left: Box::new(syn::Expr::Path(path.clone())),
                        op: assign_op.op.clone(),
                        right: assign_op.right.clone(),
                    });
                    // println!("Found compound assignment: {} = {:?}", var, right_expr);
                    return Some((var, right_expr));
                }
            }
        }
        // Handle 'let' like 'let mut sum = 0;'
        else if let syn::Stmt::Local(local) = stmt.clone() {
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                // If we have an identifier (sum)
                let var = pat_ident.ident.to_string(); // Take var identifier (string)
                if let Some((_, expr)) = &local.init {
                    return Some((var, *expr.clone())); // Return the id and literal it's initialized to (expr)
                }
            }
        }

        // println!("No valid assignment found in statement: {:#?}", stmt);
        None
    }

    /*fn print_expr_details(&self, expr: &Expr) {
        println!("Expr details: {:#?}", expr);
    }*/

    fn substitute_in_token_stream(
        &self,
        tokens: &TokenStream,
        var: &str,
        replacement: &Expr,
    ) -> TokenStream {
        // Convert the replacement expression to a string
        let replacement_string = quote! { #replacement }.to_string();
        // Parse the string back into a TokenStream
        let replacement_token_stream: TokenStream = replacement_string
            .parse()
            .expect("Failed to parse replacement string");

        tokens
            .clone()
            .into_iter()
            .flat_map(|tt| {
                match &tt {
                    TokenTree::Ident(ident) if ident.to_string() == var => replacement_token_stream
                        .clone()
                        .into_iter()
                        .collect::<Vec<_>>()
                        .into_iter(),
                    TokenTree::Group(group) => {
                        let new_stream =
                            self.substitute_in_token_stream(&group.stream(), var, replacement);
                        // println!("Replaced group: {:#?}", new_stream);
                        vec![TokenTree::Group(proc_macro2::Group::new(
                            group.delimiter(),
                            new_stream,
                        ))]
                        .into_iter()
                    }
                    TokenTree::Punct(_punct) => {
                        // println!("Punctuation: {}", punct);
                        vec![tt.clone()].into_iter()
                    }
                    TokenTree::Literal(_literal) => {
                        // println!("Literal: {}", literal);
                        vec![tt.clone()].into_iter()
                    }
                    _ => {
                        // println!("Other token: {:#?}", tt);
                        vec![tt.clone()].into_iter()
                    }
                }
            })
            .collect()
    }
}
