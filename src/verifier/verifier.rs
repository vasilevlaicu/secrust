use z3::{ast, Config, Context, Solver, SatResult};
use z3::ast::Ast;
use std::collections::HashMap;

// Z3 integer variables
fn get_or_create_var<'a>(
    ctx: &'a Context,
    name: &str,
    vars: &mut HashMap<String, ast::Int<'a>>,
) -> ast::Int<'a> {
    vars.entry(name.to_string())
        .or_insert_with(|| ast::Int::new_const(ctx, name))
        .clone()
}

// Vierfy Z3 condition
pub fn verify_condition(
    solver: &mut Solver,
    condition: &ast::Bool,
) -> bool {
    solver.push();
    solver.assert(condition);
    let result = match solver.check() {
        SatResult::Sat => {
            println!("Condition is satisfiable.\n");
            true
        }
        SatResult::Unsat => {
            println!("Condition is not satisfiable.\n");
            false
        }
        SatResult::Unknown => {
            println!("Solver could not determine satisfiability.\n");
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
    // pre ! (n > 0) >> invariant ! (i <= n + 1 && sum == (i - 1) * i / 2)
    // i <= n + 1 && sum == (i - 1) * i / 2
    let condition_path_1_pre = n.gt(&ast::Int::from_i64(&ctx, 0)); // n > 0
    let condition_path_1_invariant = i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))) /* i <= n + 1 */
        & sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2))); /* && sum == (i - 1)* i/2)*/
    let condition_path_1 = condition_path_1_pre.implies(&condition_path_1_invariant); // pre >> invariant

    println!("Verifying conditions for Path 1:");
    verify_condition(&mut solver, &condition_path_1);

    // Path 2: invariant implies !(i <= n) and postcondition
    // sum > 0 >> invariant!(i <= n + 1 && sum == (i - 1) * i / 2) >> !(i <= n) >> post!(sum == n * (n + 1) / 2)
    // i <= n + 1 && sum == (i - 1) * i / 2
    let faux = sum.gt(&ast::Int::from_i64(&ctx,0));
    let condition_path_2_invariant = i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1)))/* i<=n+1 */
        & sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2))); /* sum == (i - 1) * i / 2 */
    // !(i <= n)
    let condition_path_2_not_i_le_n = i.le(&n).not(); 
    // sum == n * (n + 1) / 2
    //let condition_path_2_post = sum._eq(&(&(n.clone() * (n.clone() + ast::Int::from_i64(&ctx, 1))) / ast::Int::from_i64(&ctx, 2))); 
    let condition_path_2_post = sum._eq(&ast::Int::from_i64(&ctx, -1000));
    // invariant >> !(i <= n) >> post
    let condition_path_2 = faux.implies(&(condition_path_2_invariant.implies(&(&condition_path_2_not_i_le_n & condition_path_2_post)))); 


    println!("\nVerifying conditions for Path 2:");
    verify_condition(&mut solver, &condition_path_2);

    // Path 3: invariant implies i <= n and next invariant
    // i + 1
    let i_plus_1 = i.clone() + ast::Int::from_i64(&ctx, 1);
    // i <= n + 1 && sum == (i - 1) * i / 2
    let condition_path_3_invariant_1 = i.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))) /* i <= n + 1 */
        & sum._eq(&(&(i.clone() - ast::Int::from_i64(&ctx, 1)) * i.clone() / ast::Int::from_i64(&ctx, 2))); /* sum == (i - 1) * i / 2 */
    // i <= n
    let condition_path_3_i_le_n = i.le(&n); 
    // (i + 1) <= n + 1 && (sum + i) == ((i + 1) - 1) * (i + 1) / 2
    let condition_path_3_invariant_2 = i_plus_1.le(&(n.clone() + ast::Int::from_i64(&ctx, 1))) /* i + 1 <= n + 1*/
        & (sum.clone() + i.clone())._eq(&(&(i_plus_1.clone() - ast::Int::from_i64(&ctx, 1)) * i_plus_1 / ast::Int::from_i64(&ctx, 2))); /* && sum + i == (i + 1 - 1) * (i+1)/2 */
    // invariant >> i <= n >> next invariant
    let condition_path_3 = condition_path_3_invariant_1
        .implies(&(&condition_path_3_i_le_n & condition_path_3_invariant_2)); 

    println!("\nVerifying conditions for Path 3:");
    verify_condition(&mut solver, &condition_path_3);
}

pub fn verify_unsat_condition() {
    // Z3 context and solver
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut solver = Solver::new(&ctx);
    let mut vars = HashMap::new();

    // integer var i
    let i = get_or_create_var(&ctx, "i", &mut vars);

    // unsatisfiable condition: i > 0 && i < 0
    let unsat_condition = i.gt(&ast::Int::from_i64(&ctx, 0)) // i > 0
        & i.lt(&ast::Int::from_i64(&ctx, 0)); // i < 0

    println!("Verifying unsatisfiable condition:");
    let result = verify_condition(&mut solver, &unsat_condition);

    if !result {
        println!("The condition is unsatisfiabled.\n");
    }
}
