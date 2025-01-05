use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, ExprForLoop, ExprReturn, ItemFn, Stmt};

#[derive(Clone, Debug)]
pub enum ConditionalExpr {
    If(Box<Expr>),
    ForLoop(ExprForLoop),
    While(Box<Expr>),
}

impl ConditionalExpr {
    pub fn to_syn_expr(&self) -> &Expr {
        match self {
            ConditionalExpr::If(expr) | ConditionalExpr::While(expr) => expr,
            ConditionalExpr::ForLoop(expr_for) => &expr_for.expr,
        }
    }
}

impl ToTokens for ConditionalExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ConditionalExpr::If(expr) => expr.to_tokens(tokens),
            ConditionalExpr::ForLoop(expr_for) => expr_for.to_tokens(tokens),
            ConditionalExpr::While(expr) => expr.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CfgNode {
    Function(String, Option<ItemFn>),
    Precondition(String, Option<Expr>),
    Postcondition(String, Option<Expr>),
    Invariant(String, Option<Expr>),
    Statement(String, Option<Stmt>),
    Cutoff(String),
    Condition(String, Option<ConditionalExpr>),
    Return(String, Option<ExprReturn>),
    MergePoint,
}

impl CfgNode {
    pub fn format_dot(&self, index: usize) -> String {
        let (label, shape) = match self {
            CfgNode::Function(func, _) => (func.clone(), "Mdiamond"),
            CfgNode::Precondition(pre, _) => (format!("Pre: {}", pre), "ellipse"),
            CfgNode::Postcondition(post, _) => (format!("Post: {}", post), "ellipse"),
            CfgNode::Invariant(inv, _) => (format!("@Inv: {}", inv), "ellipse"),
            CfgNode::Statement(stmt, _) => (stmt.clone(), "box"),
            CfgNode::Condition(cond, _) => (cond.clone(), "diamond"),
            CfgNode::Cutoff(inv) => (format!("@Cutoff {}", inv), "ellipse"),
            CfgNode::MergePoint => (String::from("Merge"), "circle"),
            CfgNode::Return(ret, _) => (format!("return: {}", ret), "ellipse"),
        };

        format!(
            "{} [label=\"{}\", shape={}]",
            index,
            self.escape_quotes_for_dot(&label),
            shape
        )
    }

    pub fn new_function(func_name: String, item_fn: ItemFn) -> Self {
        CfgNode::Function(func_name, Some(item_fn))
    }

    pub fn new_precondition(pre: String, expr: Expr) -> Self {
        CfgNode::Precondition(pre, Some(expr))
    }

    pub fn new_postcondition(post: String, expr: Expr) -> Self {
        CfgNode::Postcondition(post, Some(expr))
    }

    pub fn new_invariant(inv: String, expr: Expr) -> Self {
        CfgNode::Invariant(inv, Some(expr))
    }

    pub fn new_statement(stmt_str: String, stmt: Stmt) -> Self {
        CfgNode::Statement(stmt_str, Some(stmt))
    }

    pub fn new_cutoff(inv: String) -> Self {
        CfgNode::Cutoff(inv)
    }

    pub fn new_condition(cond: String, expr: ConditionalExpr) -> Self {
        CfgNode::Condition(cond, Some(expr))
    }

    pub fn new_return(ret: String, expr: ExprReturn) -> Self {
        CfgNode::Return(ret, Some(expr))
    }

    pub fn escape_quotes_for_dot(&self, input: &str) -> String {
        input.replace("\"", "\\\"")
    }
}
