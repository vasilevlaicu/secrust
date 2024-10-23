use z3::{ast, Config, Context, Solver, SatResult};
use z3::ast::Ast;
use std::collections::HashMap;

// get or create Z3 integer variables
fn get_or_create_var<'a>(
    ctx: &'a Context,
    name: &str,
    vars: &mut HashMap<String, ast::Int<'a>>,
) -> ast::Int<'a> {
    vars.entry(name.to_string())
        .or_insert_with(|| ast::Int::new_const(ctx, name))
        .clone()
}

pub fn verify_condition(
    solver: &mut Solver,
    condition: &ast::Bool,
) -> bool {
    solver.push();
    solver.assert(condition);
    let result = match solver.check() {
        SatResult::Sat => {
            println!("Condition is satisfiable.");
            true
        }
        SatResult::Unsat => {
            println!("Condition is not satisfiable.");
            false
        }
        SatResult::Unknown => {
            println!("Solver could not determine satisfiability.");
            false
        }
    };
    solver.pop(1);
    result
}

/// Function to verify conditions for each path
pub fn verify_conditions_for_paths() {
    // Create the Z3 context and solver
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut solver = Solver::new(&ctx);
    let mut vars = HashMap::new();

    // Define variables for the conditions
    let n = get_or_create_var(&ctx, "n", &mut vars);
    let i = get_or_create_var(&ctx, "i", &mut vars);
    let sum = get_or_create_var(&ctx, "sum", &mut vars);

    // Define the conditions for Path 1
    let condition_path_1_pre = n.gt(&ast::Int::from_i64(&ctx, 0));
    let condition_path_1_invariant = (n.clone() + ast::Int::from_i64(&ctx, 1)).ge(&ast::Int::from_i64(&ctx, 1))
        & ast::Int::from_i64(&ctx, 0)._eq(&(ast::Int::from_i64(&ctx, 1 - 1) * ast::Int::from_i64(&ctx, 1) / ast::Int::from_i64(&ctx, 2)));
    let condition_path_1 = condition_path_1_pre & condition_path_1_invariant;

    println!("Verifying conditions for Path 1:");
    verify_condition(&mut solver, &condition_path_1);

    // Define the conditions for Path 2
    let condition_path_2_invariant = i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1)))
        & sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2)));
    let condition_path_2_not_i_le_n = i.le(&n).not();
    let condition_path_2_post = sum._eq(&(&(n.clone() * (n.clone() + ast::Int::from_i64(&ctx, 1))) / ast::Int::from_i64(&ctx, 2)));
    let condition_path_2 = condition_path_2_invariant & condition_path_2_not_i_le_n & condition_path_2_post;

    println!("\nVerifying conditions for Path 2:");
    verify_condition(&mut solver, &condition_path_2);

    // Define the conditions for Path 3
    let i_plus_1 = i.clone() + ast::Int::from_i64(&ctx, 1);
    let condition_path_3_invariant_1 = i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1)))
        & sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2)));
    let condition_path_3_i_plus_1_le_n = i_plus_1.le(&n);
    let condition_path_3_invariant_2 = i_plus_1.le(&(n + ast::Int::from_i64(&ctx, 1)))
        & (sum + i)._eq(&(&(i_plus_1.clone() - ast::Int::from_i64(&ctx, 1)) * i_plus_1 / ast::Int::from_i64(&ctx, 2)));
    let condition_path_3 = condition_path_3_invariant_1 & condition_path_3_i_plus_1_le_n & condition_path_3_invariant_2;

    println!("\nVerifying conditions for Path 3:");
    verify_condition(&mut solver, &condition_path_3);
}
// implications of assume. assume () => ...