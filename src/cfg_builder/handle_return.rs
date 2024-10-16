use syn::ExprReturn;
use quote::quote;
use crate::cfg_builder::{CfgBuilder, CfgNode};

impl CfgBuilder {
    pub fn handle_return_statement(&mut self, expr_return: &ExprReturn) {
        let return_expr = expr_return.expr.as_ref().map(|expr| quote!(#expr).to_string()).unwrap_or_default();
        let return_node = self.add_node(CfgNode::new_return(return_expr, expr_return.clone()));
        self.current_node = Some(return_node);
    }
}
