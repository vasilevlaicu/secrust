use syn::{ExprMacro, punctuated::Punctuated, Expr, token::Comma};
use quote::quote;
use crate::cfg_builder::builder::CfgBuilder;
use crate::cfg_builder::node::CfgNode;

impl CfgBuilder {
    pub fn process_macro(&mut self, expr_macro: &ExprMacro) {
        let macro_name = format!("{}!", expr_macro.mac.path.segments.last().unwrap().ident);
        self.process_external_conditions(&macro_name, quote!(#expr_macro).to_string());
    }

    pub fn process_macro_call_as_function(&mut self, args: &Punctuated<Expr, Comma>, macro_name: &str) {
        let call_expression = format!("{}[{}]", macro_name, quote!(#args));
        self.process_external_conditions(macro_name, call_expression);
    }

    pub fn process_external_conditions(&mut self, name: &str, call_expression: String) {
        let external_methods = self.external_conditions.external_methods.clone();
        if let Some(external_method) = external_methods.iter().find(|m| m.name == name) {
            for pre in &external_method.preconditions {
                self.add_node(CfgNode::new_precondition(pre.clone(), Expr::Verbatim(quote!(#pre).into())));
            }
            self.add_node(CfgNode::Statement(format!("Call: {}", call_expression), None));
            for post in &external_method.postconditions {
                self.add_node(CfgNode::new_postcondition(post.clone(), Expr::Verbatim(quote!(#post).into())));
            }
        } else {
            self.add_node(CfgNode::Statement(format!("Call: {}", call_expression), None));
        }
    }
}
