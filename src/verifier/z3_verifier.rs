use crate::verifier::z3_parser;
use crate::Z3Var;
use std::collections::HashMap;
use z3::{ast, Config, Context, SatResult, Solver};

// Verify Z3 condition and print the model if satisfiable
pub fn verify_condition(
    solver: &mut Solver,
    condition: &ast::Bool,
    vars: &HashMap<String, Z3Var>,
) -> bool {
    solver.push();
    solver.assert(&condition.not()); // assert the negation for proof by contradiction
    let result = match solver.check() {
        SatResult::Unsat => {
            println!("Condition is valid (unsatisfiable when negated).\n");
            true
        }
        SatResult::Sat => {
            println!("Condition is not valid (counterexample found).\n");
            if let Some(model) = solver.get_model() {
                println!("Counterexample model assignments:");
                for (name, var) in vars {
                    let value = match var {
                        Z3Var::Int(ref int_var) => {
                            model.eval(int_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Bool(ref bool_var) => {
                            model.eval(bool_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Real(ref real_var) => {
                            model.eval(real_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::BV(ref bv_var) => {
                            model.eval(bv_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Float(ref float_var) => {
                            model.eval(float_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Array(ref array_var) => {
                            model.eval(array_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::String(ref string_var) => {
                            model.eval(string_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Set(ref set_var) => {
                            model.eval(set_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Datatype(ref datatype_var) => {
                            model.eval(datatype_var, false).map(|v| format!("{:?}", v))
                        }
                        Z3Var::Dynamic(ref dynamic_var) => {
                            model.eval(dynamic_var, false).map(|v| format!("{:?}", v))
                        }
                    };

                    if let Some(value) = value {
                        println!("{} = {}", name, value);
                    }
                }

                println!();
            }
            false
        }
        SatResult::Unknown => {
            println!("Solver could not determine validity.\n");
            false
        }
    };
    solver.pop(1);
    result
}

// Main verification function that uses the parser module
pub fn verify_str_implication(expr_str: &str) {
    // Z3 context and solver
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut solver = Solver::new(&ctx);

    // Parse and process logical proposition
    let parsed_expr = syn::parse_str::<syn::Expr>(expr_str).expect("Failed to parse expression");
    let (z3_condition, vars) = z3_parser::generate_condition_and_vars(&ctx, &parsed_expr);
    // Verify the condition
    verify_condition(&mut solver, &z3_condition, &vars);
}
