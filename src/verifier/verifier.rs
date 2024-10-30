use z3::{ast, Config, Context, Solver, SatResult};
use z3::ast::Ast;
use std::collections::HashMap;

// Get or create Z3 integer variables
fn get_or_create_var<'a>(
    ctx: &'a Context,
    name: &str,
    vars: &mut HashMap<String, ast::Int<'a>>,
) -> ast::Int<'a> {
    vars.entry(name.to_string())
        .or_insert_with(|| ast::Int::new_const(ctx, name))
        .clone()
}

// Verify Z3 condition and print the model if valid
pub fn verify_condition(
    solver: &mut Solver,
    condition: &ast::Bool,
    vars: &HashMap<String, ast::Int>,
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
                    if let Some(value) = model.eval(var) {
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

pub fn verify_conditions_for_paths() {
    // Z3 context and solver
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut solver = Solver::new(&ctx);
    let mut vars = HashMap::new();

    // Variables for the conditions
    let n = get_or_create_var(&ctx, "n", &mut vars);
    let i = get_or_create_var(&ctx, "i", &mut vars);
    let sum = get_or_create_var(&ctx, "sum", &mut vars);

    // Path 1: pre implies invariant
    // pre: n >= 0
    // invariant: i <= n + 1 && sum == (i - 1) * i / 2
    let condition_path_1_pre = z3::ast::Bool::and(&ctx, &[
        &n.ge(&ast::Int::from_i64(&ctx, 0)),  // n >= 0
    ]);
    // invariant: i >= 1 && i <= n + 1 && sum == (i - 1) * i / 2
    let one = ast::Int::from_i64(&ctx, 1);
    let zero = ast::Int::from_i64(&ctx, 0);
    let condition_path_1_invariant = z3::ast::Bool::and(&ctx, &[
        &one.le(&(n.clone() + one.clone())), // i <= n + 1
        &zero._eq(&(&(one.clone() - one.clone()) * i.clone() / ast::Int::from_i64(&ctx, 2))), // sum == (i - 1) * i / 2
    ]);

    let condition_path_1 = z3::ast::Bool::implies(&condition_path_1_pre, &condition_path_1_invariant);

    println!("Verifying conditions for Path 1:");
    verify_condition(&mut solver, &condition_path_1, &vars);

    // Path 2: invariant and !(i <= n) imply postcondition
    let condition_path_2_invariant = z3::ast::Bool::and(&ctx, &[
        &i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))), // i <= n + 1
        &sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2))), // sum == (i - 1) * i / 2
    ]);
    let condition_path_2_not_i_le_n = i.le(&n).not(); // !(i <= n)
    let condition_path_2_post = sum._eq(
        &(&(n.clone() * (n.clone() + ast::Int::from_i64(&ctx, 1))) / ast::Int::from_i64(&ctx, 2)),
    ); // sum == n * (n + 1) / 2
    let condition_path_2 = z3::ast::Bool::implies(
        &z3::ast::Bool::and(&ctx, &[
            &condition_path_2_invariant,
            &condition_path_2_not_i_le_n,
        ]),
        &condition_path_2_post,
    );

    println!("\nVerifying conditions for Path 2:");
    verify_condition(&mut solver, &condition_path_2, &vars);

    // Path 3: invariant and (i <= n) imply updated invariant
    // (i <= n + 1 && sum == (i - 1) * i / 2) >> (i <= n >> invariant ! ((i + 1) <= n + 1 && (sum + i) == ((i + 1) - 1) * (i + 1) / 2)
    let i_next = i.clone() + ast::Int::from_i64(&ctx, 1); // i + 1
    let sum_next = sum.clone() + i.clone(); // sum + 1

    let condition_path_3_invariant_current = z3::ast::Bool::and(&ctx, &[
        &i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))), // i <= n + 1
        &sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2))), // sum == (i - 1) * i/2 
    ]);

    let condition_path_3_i_le_n = i.le(&n); // i <= n

    let condition_path_3_invariant_next = z3::ast::Bool::and(&ctx, &[
        &i_next.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))), // i + 1 <= (n+1)
        // sum+1 == ((i+1)-1)*(i+1)/2
        &sum_next._eq(&(&(i_next.clone() - ast::Int::from_i64(&ctx, 1)) * i_next.clone() / ast::Int::from_i64(&ctx, 2))),
        ]);

    /*
    This doesn't work for us because we nest the implications instead of chaining them...
    let condition_path_3 = z3::ast::Bool::implies(
        &z3::ast::Bool::implies(
            &condition_path_3_invariant_current,
            &condition_path_3_i_le_n
        ), &condition_path_3_invariant_next
    );
    */

    // TODO FIX USE THIS NESTED IMPLICATION CHAIN
    let condition_path_3 = z3::ast::Bool::implies(&condition_path_3_invariant_current,
        &z3::ast::Bool::implies(
            &condition_path_3_i_le_n,
            &condition_path_3_invariant_next
        ),
    );

    println!("\nVerifying conditions for Path 3:");
    verify_condition(&mut solver, &condition_path_3, &vars);
}

pub fn verify_unsat_condition() {
    // Z3 context and solver
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut solver = Solver::new(&ctx);
    let mut vars = HashMap::new();

    // Integer variable i
    let i = get_or_create_var(&ctx, "i", &mut vars);

    // Unsatisfiable condition: i > 0 && i < 0
    let unsat_condition = z3::ast::Bool::and(&ctx, &[
        &i.gt(&ast::Int::from_i64(&ctx, 0)), // i > 0
        &i.lt(&ast::Int::from_i64(&ctx, 0)), // i < 0
    ]);

    println!("Verifying unsatisfiable condition:");
    let result = verify_condition(&mut solver, &unsat_condition, &vars);

    if !result {
        println!("The condition is unsatisfiable.\n");
    }
}

fn main() {
    verify_conditions_for_paths();
    verify_unsat_condition();
}
