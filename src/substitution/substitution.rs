use syn::{Expr, Stmt, ExprAssign, ExprBinary, ExprBlock, ExprIf, ExprCall, ExprUnary, ExprParen, Local, ExprMacro, Macro, Block};
use std::collections::HashMap;
use quote::quote;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use crate::cfg_builder::builder::CfgBuilder;
use crate::cfg_builder::node::{CfgNode};
use proc_macro2::{Span, TokenTree, TokenStream};

impl CfgBuilder {
    pub fn apply_substitution(&self, paths: &[Vec<NodeIndex>]) -> Vec<String> {
        let mut updated_postconditions = Vec::new();

        for path in paths {
            let mut variable_state = std::collections::HashMap::new();
            let mut working_condition: Option<syn::Expr> = None;

            for &node_index in path.iter().rev() {
                match &self.graph[node_index] {
                    CfgNode::Statement(stmt_str, stmt_option) => {
                        println!("Processing statement: {}", stmt_str);
                        if let Some((var, expr)) = self.parse_assignment(stmt_str) {
                            println!("Assignment found: {} = {}", var, quote! { #expr });

                            if let Some(cond) = &working_condition {
                                let new_cond = self.recursive_substitution(cond, &var, &expr);
                                println!("Substituting in condition: {} -> {}", var, quote! { #expr });
                                println!("New condition after substitution: {}", quote! { #new_cond });
                                working_condition = Some(new_cond);
                            }

                            variable_state.insert(var.clone(), expr.clone());
                        }

                        if let Some(stmt) = stmt_option {
                            if let Stmt::Local(local) = stmt {
                                let Local { pat, init, .. } = local;
                                if let Some((_, expr)) = init {
                                    if let syn::Pat::Ident(pat_ident) = pat {
                                        let var = pat_ident.ident.to_string();
                                        println!("Local declaration found: {} = {}", var, quote! { #expr });

                                        if let Some(cond) = &working_condition {
                                            let new_cond = self.recursive_substitution(cond, &var, expr);
                                            println!("Substituting in condition: {} -> {}", var, quote! { #expr });
                                            println!("New condition after substitution: {}", quote! { #new_cond });
                                            working_condition = Some(new_cond);
                                        }

                                        variable_state.insert(var.clone(), *expr.clone());
                                    }
                                }
                            }
                        }
                    },
                    CfgNode::Condition(_, Some(conditional_expr)) => {
                        let expr = conditional_expr.to_syn_expr();
                        let updated_expr = self.substitute_variables(&expr, &variable_state);
                        println!("Condition before substitution: {}", quote! { #expr });
                        println!("Condition after substitution: {}", quote! { #updated_expr });

                        working_condition = Some(if let Some(existing_cond) = &working_condition {
                            syn::parse2(quote! { #updated_expr && #existing_cond }).expect("Failed to parse conjunction")
                        } else {
                            updated_expr
                        });
                    },
                    CfgNode::Postcondition(_, Some(expr)) | CfgNode::Invariant(_, Some(expr)) => {
                        let expr = self.substitute_variables(expr, &variable_state);
                        println!("Postcondition/Invariant before substitution: {}", quote! { #expr });
                        println!("Postcondition/Invariant after substitution: {}", quote! { #expr });

                        working_condition = Some(if let Some(existing_cond) = &working_condition {
                            syn::parse2(quote! { #expr && #existing_cond }).expect("Failed to parse conjunction")
                        } else {
                            expr
                        });
                    },
                    CfgNode::Precondition(_, Some(expr)) => {
                        let expr = self.substitute_variables(expr, &variable_state);
                        println!("Precondition before substitution: {}", quote! { #expr });
                        println!("Precondition after substitution: {}", quote! { #expr });

                        working_condition = Some(if let Some(existing_cond) = &working_condition {
                            syn::parse2(quote! { #expr && #existing_cond }).expect("Failed to parse conjunction")
                        } else {
                            expr
                        });
                    },
                    _ => {}
                }
            }

            if let Some(cond) = working_condition {
                updated_postconditions.push(quote! { #cond }.to_string());
            }
        }

        updated_postconditions
    }

    pub fn recursive_substitution(&self, expr: &Expr, var: &str, replacement: &Expr) -> Expr {
        println!("Substituting in expr: {:?}", quote! {#expr});
        self.print_expr_details(expr);

        match expr {
            Expr::Path(expr_path) => {
                if expr_path.path.is_ident(var) {
                    println!("Substituting {} with {}", var, quote! {#replacement});
                    replacement.clone()
                } else {
                    expr.clone()
                }
            },
            Expr::Macro(expr_macro) => {
                let new_tokens = self.substitute_in_token_stream(&expr_macro.mac.tokens, var, replacement);
                println!("new_tokens:{:?}", new_tokens);
                Expr::Macro(ExprMacro {
                    attrs: expr_macro.attrs.clone(),
                    mac: Macro {
                        path: expr_macro.mac.path.clone(),
                        bang_token: expr_macro.mac.bang_token,
                        delimiter: expr_macro.mac.delimiter.clone(),
                        tokens: new_tokens,
                    },
                })
            },
            Expr::Assign(assign) => {
                Expr::Assign(ExprAssign {
                    attrs: assign.attrs.clone(),
                    left: Box::new(self.recursive_substitution(&assign.left, var, replacement)),
                    eq_token: assign.eq_token,
                    right: Box::new(self.recursive_substitution(&assign.right, var, replacement)),
                })
            },
            Expr::Binary(bin) => {
                println!("Binary Expression: left = {}, op = {}, right = {}", 
                    quote! {#bin.left}, quote! {#bin.op}, quote! {#bin.right});
                Expr::Binary(ExprBinary {
                    attrs: bin.attrs.clone(),
                    left: Box::new(self.recursive_substitution(&bin.left, var, replacement)),
                    op: bin.op.clone(),
                    right: Box::new(self.recursive_substitution(&bin.right, var, replacement)),
                })
            },
            Expr::Call(call) => {
                Expr::Call(ExprCall {
                    attrs: call.attrs.clone(),
                    func: Box::new(self.recursive_substitution(&call.func, var, replacement)),
                    paren_token: call.paren_token,
                    args: call.args.iter().map(|arg| self.recursive_substitution(arg, var, replacement)).collect(),
                })
            },
            Expr::Unary(unary) => {
                Expr::Unary(ExprUnary {
                    attrs: unary.attrs.clone(),
                    op: unary.op.clone(),
                    expr: Box::new(self.recursive_substitution(&unary.expr, var, replacement)),
                })
            },
            Expr::Paren(paren) => {
                Expr::Paren(ExprParen {
                    attrs: paren.attrs.clone(),
                    paren_token: paren.paren_token,
                    expr: Box::new(self.recursive_substitution(&paren.expr, var, replacement)),
                })
            },
            Expr::Block(block) => {
                Expr::Block(ExprBlock {
                    attrs: block.attrs.clone(),
                    label: block.label.clone(),
                    block: Block {
                        stmts: block.block.stmts.iter().map(|stmt| self.recursive_substitute_stmt(stmt, var, replacement)).collect(),
                        ..block.block.clone()
                    },
                })
            },
            Expr::If(expr_if) => {
                Expr::If(ExprIf {
                    cond: Box::new(self.recursive_substitution(&expr_if.cond, var, replacement)),
                    then_branch: Block {
                        stmts: expr_if.then_branch.stmts.iter().map(|stmt| self.recursive_substitute_stmt(stmt, var, replacement)).collect(),
                        ..expr_if.then_branch.clone()
                    },
                    else_branch: expr_if.else_branch.as_ref().map(|else_expr| (else_expr.0.clone(), Box::new(self.recursive_substitution(&else_expr.1, var, replacement)))),
                    ..expr_if.clone()
                })
            },
            _ => {
                println!("No substitution performed for: {}", quote! {#expr});
                expr.clone()
            },
        }
    }

    fn recursive_substitute_stmt(&self, stmt: &Stmt, var: &str, replacement: &Expr) -> Stmt {
        match stmt {
            Stmt::Expr(expr) => {
                Stmt::Expr(self.recursive_substitution(expr, var, replacement))
            },
            Stmt::Semi(expr, semi) => {
                Stmt::Semi(self.recursive_substitution(expr, var, replacement), semi.clone())
            },
            Stmt::Local(local) => {
                let init = local.init.as_ref().map(|(eq, expr)| (*eq, Box::new(self.recursive_substitution(expr, var, replacement))));
                Stmt::Local(Local {
                    pat: local.pat.clone(),
                    init,
                    attrs: local.attrs.clone(),
                    let_token: local.let_token.clone(),
                    semi_token: local.semi_token.clone(),
                })
            },
            _ => stmt.clone()
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

        println!("Parsing statement: {}", stmt);
    
        // Parse the statement into a syn::Stmt
        let stmt: syn::Stmt = match syn::parse_str(&stmt) {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to parse statement: {}", e);
                return None;
            }
        };
    
        // Debug print the parsed statement
        println!("Parsed syn::Stmt: {:#?}", &stmt);
    
        if let syn::Stmt::Expr(syn::Expr::Assign(assign)) | syn::Stmt::Semi(syn::Expr::Assign(assign), _) = stmt.clone() {
            // Handle simple assignments like `count = 0;`
            if let syn::Expr::Path(path) = *assign.left {
                if let Some(ident) = path.path.get_ident() {
                    let var = ident.to_string();
                    println!("Found assignment: {} = {:?}", var, *assign.right);
                    return Some((var, *assign.right));
                }
            }
        } else if let syn::Stmt::Expr(syn::Expr::AssignOp(assign_op)) | syn::Stmt::Semi(syn::Expr::AssignOp(assign_op), _) = stmt.clone() {
            // Handle compound assignments like `count += 1;`
            if let syn::Expr::Path(path) = *assign_op.left {
                if let Some(ident) = path.path.get_ident() {
                    let var = ident.to_string();
                    let right_expr = syn::Expr::Binary(syn::ExprBinary {
                        attrs: vec![],
                        left: Box::new(syn::Expr::Path(path.clone())),
                        op: assign_op.op.clone(),
                        right: assign_op.right.clone(),
                    });
                    println!("Found compound assignment: {} = {:?}", var, right_expr);
                    return Some((var, right_expr));
                }
            }
        }
    
        println!("No valid assignment found in statement: {:#?}", stmt);
        None
    }

    fn substitute_variables(&self, expr: &syn::Expr, variable_state: &std::collections::HashMap<String, syn::Expr>) -> syn::Expr {
        match expr {
            syn::Expr::Path(expr_path) if expr_path.path.segments.len() == 1 => {
                let var = expr_path.path.segments[0].ident.to_string();
                if let Some(replacement) = variable_state.get(&var) {
                    return replacement.clone();
                }
            }
            syn::Expr::Unary(expr_unary) => {
                let mut new_expr_unary = expr_unary.clone();
                new_expr_unary.expr = Box::new(self.substitute_variables(&expr_unary.expr, variable_state));
                return syn::Expr::Unary(new_expr_unary);
            }
            syn::Expr::Binary(expr_binary) => {
                let mut new_expr_binary = expr_binary.clone();
                new_expr_binary.left = Box::new(self.substitute_variables(&expr_binary.left, variable_state));
                new_expr_binary.right = Box::new(self.substitute_variables(&expr_binary.right, variable_state));
                return syn::Expr::Binary(new_expr_binary);
            }
            syn::Expr::Call(expr_call) => {
                let mut new_expr_call = expr_call.clone();
                new_expr_call.args = expr_call.args.iter().map(|arg| self.substitute_variables(arg, variable_state)).collect();
                return syn::Expr::Call(new_expr_call);
            }
            syn::Expr::Paren(expr_paren) => {
                let mut new_expr_paren = expr_paren.clone();
                new_expr_paren.expr = Box::new(self.substitute_variables(&expr_paren.expr, variable_state));
                return syn::Expr::Paren(new_expr_paren);
            }
            _ => {}
        }
        expr.clone()
    }

    fn print_expr_details(&self, expr: &Expr) {
        println!("Expr details: {:#?}", expr);
    }

    fn substitute_in_token_stream(&self, tokens: &TokenStream, var: &str, replacement: &Expr) -> TokenStream {
        println!("SUBSTITUTING IN MACRO");
        println!("to be replaced: {}", var);
        println!("token stream: {}", tokens);
        println!("replacement: {:#?}", quote! { #replacement });
    
        // Convert the replacement expression to a string
        let replacement_string = quote! { #replacement }.to_string();
        // Parse the string back into a TokenStream
        let replacement_token_stream: TokenStream = replacement_string.parse().expect("Failed to parse replacement string");
    
        tokens.clone().into_iter().flat_map(|tt| {
            println!("{}", tt.to_string());
            println!("var {}", var.to_string());
            match &tt {
                TokenTree::Ident(ident) if ident.to_string() == var => {
                    println!("Replacing identifier: {}", ident);
                    println!("replacement token stream: {}", replacement_token_stream);
                    replacement_token_stream.clone().into_iter().collect::<Vec<_>>().into_iter()
                },
                TokenTree::Group(group) => {
                    println!("Entering group: {}", group.stream());
                    let new_stream = self.substitute_in_token_stream(&group.stream(), var, replacement);
                    println!("Replaced group: {:#?}", new_stream);
                    vec![TokenTree::Group(proc_macro2::Group::new(group.delimiter(), new_stream))].into_iter()
                },
                TokenTree::Punct(punct) => {
                    println!("Punctuation: {}", punct);
                    vec![tt.clone()].into_iter()
                },
                TokenTree::Literal(literal) => {
                    println!("Literal: {}", literal);
                    vec![tt.clone()].into_iter()
                },
                _ => {
                    println!("Other token: {:#?}", tt);
                    vec![tt.clone()].into_iter()
                }
            }
        }).collect()
    }
}
