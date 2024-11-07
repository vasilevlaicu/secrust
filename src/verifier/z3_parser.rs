use z3::{ast, Context};
use z3::ast::Ast;   
use syn::{Expr, UnOp, ExprPath, ExprLit, ExprMacro, ExprBinary, ExprParen, ExprUnary, BinOp};
use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;
use z3::ast::Bool;

// Enum to represent different Z3 variable types
// (just using INT and bool for now)
#[derive(Clone)]
#[derive(Debug)]
pub enum Z3Var<'ctx> {
    Int(ast::Int<'ctx>),
    Bool(ast::Bool<'ctx>),
    Real(ast::Real<'ctx>),
    BV(ast::BV<'ctx>),
    Float(ast::Float<'ctx>),
    Array(ast::Array<'ctx>),
    String(ast::String<'ctx>),
    Set(ast::Set<'ctx>),
    Datatype(ast::Datatype<'ctx>),
    Dynamic(ast::Dynamic<'ctx>),
}

#[derive(Debug, Clone)]
struct ImplicationPlaceholder<'a> {
    chain: Vec<ast::Bool<'a>>, // Store translated Z3 Bool expressions
}

impl<'a> ImplicationPlaceholder<'a> {
    fn new() -> Self {
        Self { chain: Vec::new() }
    }

    fn add_argument(&mut self, arg: ast::Bool<'a>) {
        self.chain.push(arg);
    }

    /// Converts the chain into nested Z3 implications
    fn to_z3_implies(self, ctx: &'a Context) -> ast::Bool<'a> {
        self.chain.into_iter().rev().reduce(|acc, expr| ast::Bool::implies(&expr, &acc))
            .expect("ImplicationPlaceholder must have at least one argument")
    }
}


// Main function to generate Z3 condition and variables HashMap
pub fn generate_condition_and_vars<'a>(
    ctx: &'a Context,
    expr: &Expr,
) -> (ast::Bool<'a>, HashMap<String, Z3Var<'a>>) {
    let mut vars = HashMap::new();
    //println!("Whole SYN AST: {:?}", expr);
    let z3_condition_var = generate_z3_ast(ctx, expr, &mut vars);

    // Ensure the condition is returned as a Bool, converting if necessary
    let z3_condition = match z3_condition_var {
        Z3Var::Bool(b) => b,
        _ => panic!("Expected Bool condition, found different type"),
    };

    // Post-process the AST to handle implication placeholders
    let z3_condition = post_process_implications(&z3_condition, ctx);

    /*println!("Variables in the condition:");
    for (name, var) in &vars {
        match var {
            Z3Var::Int(int_var) => println!("{} = Int({})", name, int_var.to_string()),
            Z3Var::Bool(bool_var) => println!("{} = Bool({})", name, bool_var.to_string()),
            Z3Var::Real(real_var) => println!("{} = Real({})", name, real_var.to_string()),
            Z3Var::BV(bv_var) => println!("{} = BV({})", name, bv_var.to_string()),
            Z3Var::Float(float_var) => println!("{} = Float({})", name, float_var.to_string()),
            Z3Var::Array(array_var) => println!("{} = Array({})", name, array_var.to_string()),
            Z3Var::String(string_var) => println!("{} = String({})", name, string_var.to_string()),
            Z3Var::Set(set_var) => println!("{} = Set({})", name, set_var.to_string()),
            Z3Var::Datatype(datatype_var) => println!("{} = Datatype({})", name, datatype_var.to_string()),
            Z3Var::Dynamic(dynamic_var) => println!("{} = Dynamic({})", name, dynamic_var.to_string()),
        }
    }*/
    println!();
    println!("Generated Z3 Condition:\n{}\n", z3_condition.to_string());
    (z3_condition, vars)
}

fn generate_z3_ast<'a>(
    ctx: &'a Context,
    expr: &Expr,
    vars: &mut HashMap<String, Z3Var<'a>>,
) -> Z3Var<'a> {
    match expr {
        Expr::Macro(ExprMacro { mac, .. }) => {
            let macro_name = mac.path.segments.last().expect("Expected macro name").ident.to_string();
            if ["invariant", "pre", "post"].contains(&macro_name.as_str()) {
                if let Ok(arg_expr) = syn::parse2::<Expr>(mac.tokens.clone()) {
                    return generate_z3_ast(ctx, &arg_expr, vars);
                } else {
                    panic!("Failed to parse macro argument expression");
                }
            } else {
                panic!("Unsupported macro: {}", macro_name);
            }
        }
        Expr::Lit(ExprLit { lit, .. }) => match lit {
            syn::Lit::Int(lit_int) => {
                let int_value = lit_int.base10_parse::<i64>().expect("Expected integer literal");
                Z3Var::Int(ast::Int::from_i64(ctx, int_value))
            }
            syn::Lit::Bool(lit_bool) => {
                Z3Var::Bool(ast::Bool::from_bool(ctx, lit_bool.value))
            }
            _ => panic!("Unsupported literal type"),
        },
        Expr::Paren(ExprParen { expr, .. }) => {
            generate_z3_ast(ctx, expr, vars)
        }
        Expr::Path(ExprPath { path, .. }) => {
            if let Some(ident) = path.get_ident() {
                let var_name = ident.to_string();
                get_or_create_var(ctx, &var_name, vars)
            } else {
                panic!("Unsupported path expression");
            }
        }
        Expr::Unary(ExprUnary { op, expr, .. }) => match op {
            syn::UnOp::Not(_) => {
                let inner_ast = generate_z3_ast(ctx, expr, vars);
                match inner_ast {
                    Z3Var::Bool(inner_bool) => Z3Var::Bool(inner_bool.not()),
                    _ => panic!("Expected Bool type for Not operation"),
                }
            }
            _ => panic!("Unsupported unary operator: {:?}", op),
        },
        Expr::Binary(ExprBinary { left, op, right, .. }) => {
            let left_ast = generate_z3_ast(ctx, left, vars);
            let right_ast = generate_z3_ast(ctx, right, vars);

            match op {
                BinOp::And(_) => {
                    if let (Z3Var::Bool(left_bool), Z3Var::Bool(right_bool)) = (left_ast, right_ast) {
                        Z3Var::Bool(ast::Bool::and(ctx, &[&left_bool, &right_bool]))
                    } else {
                        panic!("Expected Bool types for And operation");
                    }
                },
                BinOp::Or(_) => {
                    if let (Z3Var::Bool(left_bool), Z3Var::Bool(right_bool)) = (left_ast, right_ast) {
                        Z3Var::Bool(ast::Bool::or(ctx, &[&left_bool, &right_bool]))
                    } else {
                        panic!("Expected Bool types for Or operation");
                    }
                }
                BinOp::Eq(_) => match (left_ast, right_ast) {
                    (Z3Var::Int(left_int), Z3Var::Int(right_int)) => Z3Var::Bool(left_int._eq(&right_int)),
                    (Z3Var::Bool(left_bool), Z3Var::Bool(right_bool)) => Z3Var::Bool(left_bool._eq(&right_bool)),
                    _ => panic!("Unsupported types for Eq operation"),
                },
                BinOp::Le(_) => {
                    match (left_ast, right_ast) {
                        (Z3Var::Int(left_int), Z3Var::Int(right_int)) => {
                            // println!("Attempting Le operation: left = {:?}, right = {:?}", left_int, right_int);
                            Z3Var::Bool(left_int.le(&right_int))
                        }
                        (left, right) => {
                            println!(
                                "Expected Int types for Le operation, found incompatible types: left = {:?}, right = {:?}",
                                left, right
                            );
                            panic!("Comparison operations require Int types only.");
                        }
                    }
                },                                                                  
                BinOp::Ge(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Bool(left_int.ge(&right_int))
                    } else {
                        panic!("Expected Int types for Ge operation");
                    }
                }
                BinOp::Lt(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Bool(left_int.lt(&right_int))
                    } else {
                        panic!("Expected Int types for Lt operation");
                    }
                }
                BinOp::Gt(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Bool(left_int.gt(&right_int))
                    } else {
                        panic!("Expected Int types for Gt operation");
                    }
                }
                BinOp::Add(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Int(left_int.add(&right_int))
                    } else {
                        panic!("Expected Int types for Add operation");
                    }
                }
                BinOp::Sub(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Int(left_int.sub(&right_int))
                    } else {
                        panic!("Expected Int types for Sub operation");
                    }
                }
                BinOp::Mul(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Int(left_int.mul(&right_int))
                    } else {
                        panic!("Expected Int types for Mul operation");
                    }
                }
                BinOp::Div(_) => {
                    if let (Z3Var::Int(left_int), Z3Var::Int(right_int)) = (left_ast, right_ast) {
                        Z3Var::Int(left_int.div(&right_int))
                    } else {
                        panic!("Expected Int types for Div operation");
                    }
                }
                BinOp::Shr(_) => {
                    // println!("Detected `>>` operation in Syn AST:");
                    // println!("Left: {:?}", left);
                    // println!("Right: {:?}", right);
                
                    let mut placeholder = ImplicationPlaceholder::new();
                
                    // Helper function to traverse and extract chained implications
                    fn extract_chain<'a>(
                        ctx: &'a Context,
                        expr: &Expr,
                        vars: &mut HashMap<String, Z3Var<'a>>,
                        placeholder: &mut ImplicationPlaceholder<'a>,
                    ) {
                        if let Expr::Binary(ExprBinary { left, op, right, .. }) = expr {
                            if matches!(op, BinOp::Shr(_)) {
                                // If the left side is also a `>>`, traverse it recursively
                                extract_chain(ctx, left, vars, placeholder);
                
                                // Process the right side and add it to the placeholder
                                if let Z3Var::Bool(right_bool) = generate_z3_ast(ctx, right, vars) {
                                    placeholder.add_argument(right_bool);
                                } else {
                                    panic!("Expected Bool type for right operand of `>>`");
                                }
                                return;
                            }
                        }
                
                        // If it's not a chain, process it as a standalone expression
                        if let Z3Var::Bool(expr_bool) = generate_z3_ast(ctx, expr, vars) {
                            placeholder.add_argument(expr_bool);
                        } else {
                            panic!("Expected Bool type for chain element");
                        }
                    }
                
                    // Extract the left side chain
                    extract_chain(ctx, left, vars, &mut placeholder);
                
                    // Process the right side of the current `>>` operation
                    if let Z3Var::Bool(right_bool) = generate_z3_ast(ctx, right, vars) {
                        placeholder.add_argument(right_bool);
                    } else {
                        println!("Left operand: {:?}", left);
                        panic!("Expected Bool type for right operand of top-level `>>`: {:?}", right);
                    }
                
                    // Return the placeholder as a `Z3Var::Bool`
                    Z3Var::Bool(placeholder.to_z3_implies(ctx))
                }                                                                                                                                                                               
                _ => panic!("Unsupported binary operator: {:?}", op),
            }
        }
        other => {
            println!("Encountered unsupported logical expression type: {:?}", other);
            panic!("Unsupported logical expression");
        }
    }
}

fn post_process_implications<'a>(
    expr: &ast::Bool<'a>,
    ctx: &'a Context,
) -> ast::Bool<'a> {
    if let Some(placeholder) = extract_implication_placeholder(expr) {
        // Print the chain for debugging
        /*println!("Implication chain detected:");
        for (i, implication) in placeholder.chain.iter().enumerate() {
            println!("  [{}]: {}", i, implication.to_string());
        }*/

        // Convert the placeholder to nested implications
        return placeholder.to_z3_implies(ctx);
    }

    // Recursively process left and right if this is an implication
    if expr.decl().kind() == z3::DeclKind::IMPLIES {
        let args = expr.children();

        if args.len() == 2 {
            let left = post_process_implications(
                &args[0].clone().try_into().expect("Expected Bool type"),
                ctx,
            );
            let right = post_process_implications(
                &args[1].clone().try_into().expect("Expected Bool type"),
                ctx,
            );

            println!(
                "Processing implication: {} => {}",
                left.to_string(),
                right.to_string()
            );

            return ast::Bool::implies(&left, &right);
        }
    }

    println!("Non-implication or terminal node: {}", expr.to_string());

    expr.clone() // Return the original expression if no placeholder or processing needed
}





fn extract_implication_placeholder<'a>(
    expr: &ast::Bool<'a>,
) -> Option<ImplicationPlaceholder<'a>> {
    if expr.decl().kind() == z3::DeclKind::IMPLIES {
        let args = expr.children();

        let mut placeholder = ImplicationPlaceholder::new();

        if let Some(left_dynamic) = args.get(0) {
            let left = left_dynamic.clone().try_into().ok()?;
            placeholder.add_argument(left);
        }

        if let Some(right_dynamic) = args.get(1) {
            let right = right_dynamic.clone().try_into().ok()?;
            placeholder.add_argument(right);
        }

        return Some(placeholder);
    }

    None
}






fn try_as_implies<'a>(expr: &ast::Bool<'a>) -> Option<(ast::Bool<'a>, ast::Bool<'a>)> {
    let decl = expr.decl(); // Get the declaration (function/operator)
    if decl.kind() == z3::DeclKind::IMPLIES {
        let args = expr.children(); // Use `children` to fetch the arguments
        if args.len() == 2 {
            let lhs = args[0].clone().try_into().ok()?; // Ensure it's a `Bool`
            let rhs = args[1].clone().try_into().ok()?;
            return Some((lhs, rhs));
        }
    }
    None
}


// Generate Z3 Int AST from arithmetic expressions
fn generate_z3_ast_expr<'a>(
    ctx: &'a Context,
    expr: &Expr,
    vars: &mut HashMap<String, Z3Var<'a>>,
) -> Z3Var<'a> {
    match expr {
        Expr::Binary(ExprBinary { left, op, right, .. }) => {
            let left_ast = generate_z3_ast_expr(ctx, left, vars);
            let right_ast = generate_z3_ast_expr(ctx, right, vars);

            match op {
                BinOp::Add(_) => Z3Var::Int(left_ast.as_int().add(right_ast.as_int())),
                BinOp::Sub(_) => Z3Var::Int(left_ast.as_int().sub(right_ast.as_int())),
                BinOp::Mul(_) => Z3Var::Int(left_ast.as_int().mul(right_ast.as_int())),
                BinOp::Div(_) => Z3Var::Int(left_ast.as_int().div(right_ast.as_int())),
                unsupported_expr => {
                    println!("Unsupported integer operation");
                    panic!("operation type: {:?}", unsupported_expr);
                }
            }
        }
        Expr::Paren(ExprParen { expr, .. }) => {
            // Process the inner expression of parentheses
            generate_z3_ast_expr(ctx, expr, vars)
        }
        Expr::Lit(lit) => {
            // Handle integer literals
            if let syn::Lit::Int(int_lit) = &lit.lit {
                let value = int_lit.base10_parse::<i64>().expect("Failed to parse integer literal");
                Z3Var::Int(ast::Int::from_i64(ctx, value))
            } else {
                panic!("Unsupported literal type")
            }
        }
        Expr::Path(expr_path) => {
            let var_name = expr_path.path.get_ident().expect("Expected identifier").to_string();
            get_or_create_var(ctx, &var_name, vars)
        }
        unsupported_expr => {
            // Print unsupported integer expression type and panic
            println!("Encountered unsupported integer expression type: {:?}", unsupported_expr);
            panic!("Unsupported integer expression");
        }
    }
}



// Helper function to create or retrieve Z3 variables
fn get_or_create_var<'a>(
    ctx: &'a Context,
    name: &str,
    vars: &mut HashMap<String, Z3Var<'a>>,
) -> Z3Var<'a> {
    vars.entry(name.to_string())
        .or_insert_with(|| Z3Var::Int(ast::Int::new_const(ctx, name)))
        .clone()
}

// Helper methods for `Z3Var` to return specific types
impl<'ctx> Z3Var<'ctx> {
    fn as_bool(&self) -> &ast::Bool<'ctx> {
        if let Z3Var::Bool(bool_var) = self {
            bool_var
        } else {
            panic!("Expected Bool type, but found a different type.")
        }
    }

    fn as_int(&self) -> &ast::Int<'ctx> {
        if let Z3Var::Int(int_var) = self {
            int_var
        } else {
            panic!("Expected Int type, but found a different type.")
        }
    }
}