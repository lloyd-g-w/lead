use std::collections::HashSet;

use crate::{
    cell::CellRef,
    common::{LeadErr, LeadErrCode, Literal},
    evaluator::{Eval, evaluate_expr},
    grid::Grid,
    parser::Expr,
};


pub fn eval_n_arg_numeric(
    n: usize,
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: fn(Vec<f64>) -> f64,
    func_name: String,
) -> Result<Eval, LeadErr> {
    if args.len() != n {
        return Err(LeadErr {
            title: "Evaluation error.".into(),
            desc: format!("{func_name} function requires {n} argument(s)."),
            code: LeadErrCode::Invalid,
        });
    }

    let err = LeadErr {
        title: "Evaluation error.".into(),
        desc: format!("{func_name} function requires numeric argument(s)."),
        code: LeadErrCode::TypeErr,
    };

    let mut numbers = Vec::with_capacity(n);

    for arg in args {
        match evaluate_expr(arg, precs, grid)? {
            Eval::Literal(Literal::Number(num)) => numbers.push(num),
            Eval::CellRef { eval, .. } => match *eval {
                Eval::Literal(Literal::Number(num)) => numbers.push(num),
                _ => return Err(err.clone()),
            },
            _ => return Err(err.clone()),
        }
    }

    Ok(Eval::Literal(Literal::Number(func(numbers))))
}

// This is a utility function that filters out and error handles all non literal numbers or unset
// eval types and handles ranges
pub fn eval_numeric_func(
    args: &Vec<Expr>,
    precs: &mut HashSet<CellRef>,
    grid: Option<&Grid>,
    func: impl Fn(&[f64]) -> Result<f64, LeadErr>,
    func_name: &str,
) -> Result<Eval, LeadErr> {
    let mut numbers = Vec::new();

    for arg in args {
        let eval = evaluate_expr(arg, precs, grid)?;

        match eval {
            Eval::Literal(Literal::Number(n)) => numbers.push(n),
            Eval::Unset => {} // skip
            Eval::Range(range) => {
                for cell in range {
                    match cell {
                        Eval::CellRef { eval: boxed, .. } => match *boxed {
                            Eval::Literal(Literal::Number(n)) => numbers.push(n),
                            Eval::Unset => {}
                            _ => {
                                return Err(LeadErr {
                                    title: "Evaluation error.".into(),
                                    desc: format!(
                                        "Expected numeric types for {func_name} function."
                                    ),
                                    code: LeadErrCode::Unsupported,
                                });
                            }
                        },
                        _ => {
                            return Err(LeadErr {
                                title: "Evaluation error.".into(),
                                desc: format!(
                                    "Found non-cellref in RANGE during {func_name} evaluation."
                                ),
                                code: LeadErrCode::Server,
                            });
                        }
                    }
                }
            }
            _ => {
                return Err(LeadErr {
                    title: "Evaluation error.".into(),
                    desc: format!("Expected numeric types for {func_name} function."),
                    code: LeadErrCode::Unsupported,
                });
            }
        }
    }

    let res = func(&numbers)?;
    Ok(Eval::Literal(Literal::Number(res)))
}
