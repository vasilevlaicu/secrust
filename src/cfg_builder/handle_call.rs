use crate::cfg_builder::builder::CfgBuilder;
use crate::cfg_builder::node::CfgNode;
use quote::quote;
use syn::{visit::Visit, Expr, ExprCall, ExprMethodCall, Stmt};

impl CfgBuilder {
    pub fn handle_call(&mut self, expr_call: &ExprCall) {
        if let Expr::Path(expr_path) = &*expr_call.func {
            if let Some(segment) = expr_path.path.segments.last() {
                if segment.ident == "vec" {
                    // Handle vec![] macro call here
                    self.process_macro_call_as_function(&expr_call.args, "vec!");
                }
            }
        }
        // Visit arguments of the call
        for arg in &expr_call.args {
            self.visit_expr(arg);
        }
    }

    pub fn handle_method_call(&mut self, expr_method_call: &ExprMethodCall) {
        let method_name = expr_method_call.method.to_string();
        let maybe_external_method = self
            .external_conditions
            .external_methods
            .iter()
            .find(|m| m.name == method_name)
            .cloned();

        if let Some(external_method) = maybe_external_method {
            // Add preconditions before the method call
            for pre in external_method.preconditions {
                self.add_node(CfgNode::new_precondition(
                    pre,
                    Expr::MethodCall(expr_method_call.clone()),
                ));
            }

            // Add the full method call expression
            let call_expression = quote!(#expr_method_call).to_string();
            let call_description = format!("Call: {}", Self::clean_up_formatting(&call_expression));
            let call_statement = Stmt::Expr(Expr::MethodCall(expr_method_call.clone()));
            self.add_node(CfgNode::new_statement(call_description, call_statement));

            // Add postconditions after the method call
            for post in external_method.postconditions {
                self.add_node(CfgNode::new_postcondition(
                    post,
                    Expr::MethodCall(expr_method_call.clone()),
                ));
            }
        } else {
            // If no external conditions match, add the method call as a single node
            let call_expression = quote!(#expr_method_call).to_string();
            let call_description = format!("Call: {}", Self::clean_up_formatting(&call_expression));
            let call_statement = Stmt::Expr(Expr::MethodCall(expr_method_call.clone()));
            self.add_node(CfgNode::new_statement(call_description, call_statement));
        }
    }
}
